# Pattern 44: File Upload & Download Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Status**: Active

---

## Table of Contents

1. [Overview](#overview)
2. [File Upload Strategies](#file-upload-strategies)
3. [Local Storage](#local-storage)
4. [Cloud Storage (S3)](#cloud-storage-s3)
5. [File Validation & Security](#file-validation--security)
6. [Streaming Large Files](#streaming-large-files)
7. [Progress Tracking](#progress-tracking)
8. [Image Processing](#image-processing)
9. [Download Patterns](#download-patterns)
10. [Best Practices](#best-practices)

---

## Overview

File handling is critical for document management, user avatars, invoice attachments, and other business needs in a PSA platform.

### Use Cases in WellOS

- **User Avatars**: Profile pictures
- **Invoice Attachments**: PDF invoices, receipts
- **Project Documents**: Contracts, SOWs, deliverables
- **Time Entry Attachments**: Screenshots, work evidence
- **Expense Receipts**: Scanned receipts, photos
- **Report Exports**: CSV, PDF reports

---

## File Upload Strategies

### 1. Multipart Form Data (Small Files < 10MB)

**Axum Handler**:

```rust
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
struct FileResponse {
    filename: String,
    original_name: String,
    size: u64,
    mime_type: String,
}

async fn upload_file(
    mut multipart: Multipart,
) -> Result<Json<FileResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = field.bytes().await.unwrap();

            if data.is_empty() {
                return Err(StatusCode::BAD_REQUEST);
            }

            return Ok(Json(FileResponse {
                filename: filename.clone(),
                original_name: filename,
                size: data.len() as u64,
                mime_type: content_type,
            }));
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

async fn upload_multiple_files(
    mut multipart: Multipart,
) -> Result<Json<Vec<FileResponse>>, StatusCode> {
    let mut files = Vec::new();
    const MAX_FILES: usize = 10;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if files.len() >= MAX_FILES {
            break;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap();

        files.push(FileResponse {
            filename: filename.clone(),
            original_name: filename,
            size: data.len() as u64,
            mime_type: content_type,
        });
    }

    if files.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(files))
}

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/files/upload", post(upload_file))
        .route("/api/v1/files/upload-multiple", post(upload_multiple_files))
}
```

**React Client**:

```typescript
function FileUpload() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (files && files.length > 0) {
      setSelectedFile(files[0]);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    setUploading(true);

    const formData = new FormData();
    formData.comend('file', selectedFile);

    try {
      const response = await fetch('http://localhost:4001/api/v1/files/upload', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('accessToken')}`,
        },
        body: formData,
      });

      const data = await response.json();
      console.log('File uploaded:', data);
    } catch (error) {
      console.error('Upload failed:', error);
    } finally {
      setUploading(false);
    }
  };

  return (
    <div>
      <input type="file" onChange={handleFileChange} />
      <button onClick={handleUpload} disabled={!selectedFile || uploading}>
        {uploading ? 'Uploading...' : 'Upload'}
      </button>
    </div>
  );
}
```

### 2. Chunked Upload (Large Files > 10MB)

**Server**:

```rust
use axum::{
    extract::{Json, Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct ChunkMetadata {
    chunk_index: usize,
    total_chunks: usize,
    filename: String,
    upload_id: String,
}

#[derive(Debug)]
struct UploadSession {
    chunks: Vec<Option<Vec<u8>>>,
    total_chunks: usize,
    filename: String,
}

#[derive(Clone)]
struct AppState {
    upload_sessions: Arc<RwLock<HashMap<String, UploadSession>>>,
}

#[derive(Serialize)]
struct ChunkResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    received_chunks: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_chunks: Option<usize>,
}

async fn upload_chunk(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ChunkResponse>, StatusCode> {
    let mut chunk_data: Option<Vec<u8>> = None;
    let mut metadata: Option<ChunkMetadata> = None;

    // Extract chunk data and metadata from multipart
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if name == "chunk" {
            chunk_data = Some(field.bytes().await.unwrap().to_vec());
        } else if name == "metadata" {
            let text = field.text().await.unwrap();
            metadata = Some(serde_json::from_str(&text).unwrap());
        }
    }

    let chunk = chunk_data.ok_or(StatusCode::BAD_REQUEST)?;
    let meta = metadata.ok_or(StatusCode::BAD_REQUEST)?;

    let mut sessions = state.upload_sessions.write().await;

    // Initialize upload session if it doesn't exist
    if !sessions.contains_key(&meta.upload_id) {
        sessions.insert(
            meta.upload_id.clone(),
            UploadSession {
                chunks: vec![None; meta.total_chunks],
                total_chunks: meta.total_chunks,
                filename: meta.filename.clone(),
            },
        );
    }

    let session = sessions.get_mut(&meta.upload_id).unwrap();
    session.chunks[meta.chunk_index] = Some(chunk);

    // Check if all chunks received
    let all_chunks_received = session.chunks.iter().all(|c| c.is_some());

    if all_chunks_received {
        // Combine chunks
        let complete_file: Vec<u8> = session
            .chunks
            .iter()
            .filter_map(|c| c.as_ref())
            .flat_map(|c| c.iter())
            .copied()
            .collect();

        let filepath = save_file(&complete_file, &session.filename).await?;
        let size = complete_file.len();

        // Clean up session
        sessions.remove(&meta.upload_id);

        Ok(Json(ChunkResponse {
            status: "complete".to_string(),
            filepath: Some(filepath),
            size: Some(size),
            received_chunks: None,
            total_chunks: None,
        }))
    } else {
        let received = session.chunks.iter().filter(|c| c.is_some()).count();

        Ok(Json(ChunkResponse {
            status: "incomplete".to_string(),
            filepath: None,
            size: None,
            received_chunks: Some(received),
            total_chunks: Some(meta.total_chunks),
        }))
    }
}

async fn save_file(buffer: &[u8], filename: &str) -> Result<String, StatusCode> {
    let timestamp = chrono::Utc::now().timestamp();
    let filepath = format!("/uploads/{}-{}", timestamp, filename);

    fs::write(&filepath, buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(filepath)
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/files/upload-chunk", post(upload_chunk))
        .with_state(state)
}
```

**Client**:

```typescript
async function uploadLargeFile(file: File) {
  const CHUNK_SIZE = 1024 * 1024; // 1MB chunks
  const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
  const uploadId = `${Date.now()}-${Math.random()}`;

  for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
    const start = chunkIndex * CHUNK_SIZE;
    const end = Math.min(start + CHUNK_SIZE, file.size);
    const chunk = file.slice(start, end);

    const formData = new FormData();
    formData.comend('chunk', chunk);
    formData.comend('chunkIndex', chunkIndex.toString());
    formData.comend('totalChunks', totalChunks.toString());
    formData.comend('filename', file.name);
    formData.comend('uploadId', uploadId);

    const response = await fetch('http://localhost:4001/api/v1/files/upload-chunk', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: formData,
    });

    const data = await response.json();

    if (data.status === 'complete') {
      console.log('Upload complete:', data.filepath);
      return data;
    }

    // Update progress
    const progress = ((chunkIndex + 1) / totalChunks) * 100;
    console.log(`Upload progress: ${progress.toFixed(2)}%`);
  }
}
```

---

## Local Storage

### 1. File Upload Configuration

```rust
use axum::{
    body::Bytes,
    extract::Multipart,
    http::StatusCode,
};
use std::path::{Path, PathBuf};
use tokio::fs;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "application/pdf",
];

struct FileUploadConfig {
    upload_dir: PathBuf,
    max_size: usize,
    allowed_types: Vec<String>,
}

impl FileUploadConfig {
    fn new(upload_dir: impl AsRef<Path>) -> Self {
        Self {
            upload_dir: upload_dir.as_ref().to_path_buf(),
            max_size: MAX_FILE_SIZE,
            allowed_types: ALLOWED_MIME_TYPES.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn validate_file(&self, mime_type: &str, size: usize) -> Result<(), String> {
        if size > self.max_size {
            return Err(format!("File size exceeds maximum of {} bytes", self.max_size));
        }

        if !self.allowed_types.contains(&mime_type.to_string()) {
            return Err(format!("Invalid file type: {}", mime_type));
        }

        Ok(())
    }

    fn generate_filename(&self, original_name: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let random = rand::random::<u32>();
        let ext = Path::new(original_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");

        format!("file-{}-{}.{}", timestamp, random, ext)
    }
}
```

### 2. File Entity

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileId(Uuid);

impl FileId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn value(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct File {
    id: FileId,
    filename: String,
    original_name: String,
    mime_type: String,
    size: usize,
    path: String,
    uploaded_by: String,
    uploaded_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl File {
    pub fn create(
        filename: String,
        original_name: String,
        mime_type: String,
        size: usize,
        path: String,
        uploaded_by: String,
    ) -> Self {
        Self {
            id: FileId::new(),
            filename,
            original_name,
            mime_type,
            size,
            path,
            uploaded_by,
            uploaded_at: Utc::now(),
            deleted_at: None,
        }
    }

    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    pub fn is_pdf(&self) -> bool {
        self.mime_type == "application/pdf"
    }

    // Getters
    pub fn id(&self) -> &FileId {
        &self.id
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
```

---

## Cloud Storage (S3)

### 1. AWS S3 Setup

```bash
cargo add aws-sdk-s3 aws-config tokio
```

```rust
use aws_sdk_s3::{
    Client,
    presigning::PresigningConfig,
    primitives::ByteStream,
};
use aws_config::BehaviorVersion;
use std::time::Duration;

#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket_name: String,
}

impl S3Service {
    pub async fn new(bucket_name: String) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        Self {
            client,
            bucket_name,
        }
    }

    pub async fn upload_file(
        &self,
        data: Vec<u8>,
        key: String,
        content_type: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .content_type(&content_type)
            .send()
            .await?;

        let url = format!(
            "https://{}.s3.amazonaws.com/{}",
            self.bucket_name, key
        );

        Ok(url)
    }

    pub async fn get_signed_url(
        &self,
        key: &str,
        expires_in_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let presigning_config = PresigningConfig::expires_in(
            Duration::from_secs(expires_in_secs)
        )?;

        let presigned = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    pub async fn delete_file(
        &self,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_file_stream(
        &self,
        key: &str,
    ) -> Result<ByteStream, Box<dyn std::error::Error>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await?;

        Ok(response.body)
    }
}
```

### 2. Upload to S3 Handler

```rust
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    s3_service: S3Service,
    file_repository: Arc<dyn FileRepository>,
}

#[derive(Serialize)]
struct UploadResponse {
    id: String,
    url: String,
    filename: String,
    size: usize,
}

#[derive(Serialize)]
struct DownloadUrlResponse {
    url: String,
}

async fn upload_to_s3(
    State(state): State<AppState>,
    user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            filename = Some(field.file_name().unwrap_or("unknown").to_string());
            content_type = Some(
                field.content_type().unwrap_or("application/octet-stream").to_string()
            );
            file_data = Some(field.bytes().await.unwrap().to_vec());
        }
    }

    let data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let original_name = filename.ok_or(StatusCode::BAD_REQUEST)?;
    let mime_type = content_type.ok_or(StatusCode::BAD_REQUEST)?;

    // Generate S3 key
    let timestamp = chrono::Utc::now().timestamp_millis();
    let key = format!("{}/{}-{}", user.organization_id, timestamp, original_name);

    // Upload to S3
    let url = state.s3_service
        .upload_file(data.clone(), key.clone(), mime_type.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save metadata to database
    let file_entity = File::create(
        original_name.clone(),
        original_name.clone(),
        mime_type,
        data.len(),
        key,
        user.user_id.clone(),
    );

    state.file_repository
        .save(&file_entity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UploadResponse {
        id: file_entity.id().value(),
        url,
        filename: original_name,
        size: data.len(),
    }))
}

async fn get_download_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DownloadUrlResponse>, StatusCode> {
    let file = state.file_repository
        .find_by_id(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let signed_url = state.s3_service
        .get_signed_url(file.path(), 300) // 5 minutes
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DownloadUrlResponse { url: signed_url }))
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/files/upload", post(upload_to_s3))
        .route("/api/v1/files/:id/download-url", get(get_download_url))
        .with_state(state)
}
```

---

## File Validation & Security

### 1. Comprehensive Validation

```typescript
import { extname } from 'path';
import * as fileType from 'file-type';

export class FileValidator {
  private static readonly ALLOWED_MIME_TYPES = new Set([
    'image/jpeg',
    'image/png',
    'image/gif',
    'image/webp',
    'application/pdf',
    'application/msword',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'application/vnd.ms-excel',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  ]);

  private static readonly MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB

  static async validate(file: Express.Multer.File): Promise<void> {
    // 1. Check file size
    if (file.size > this.MAX_FILE_SIZE) {
      throw new BadRequestException(
        `File size exceeds maximum allowed size of ${this.MAX_FILE_SIZE / 1024 / 1024}MB`,
      );
    }

    // 2. Check MIME type from client
    if (!this.ALLOWED_MIME_TYPES.has(file.mimetype)) {
      throw new BadRequestException(`File type not allowed: ${file.mimetype}`);
    }

    // 3. Verify actual file type (prevent MIME type spoofing)
    const detectedType = await fileType.fromBuffer(file.buffer);

    if (!detectedType || !this.ALLOWED_MIME_TYPES.has(detectedType.mime)) {
      throw new BadRequestException(
        'File type mismatch. The actual file type does not match the declared type.',
      );
    }

    // 4. Validate file extension
    const ext = extname(file.originalname).toLowerCase();
    const allowedExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.pdf', '.doc', '.docx', '.xls', '.xlsx'];

    if (!allowedExtensions.includes(ext)) {
      throw new BadRequestException(`File extension not allowed: ${ext}`);
    }

    // 5. Scan for malware (optional - integrate with ClamAV or similar)
    // await this.scanForMalware(file.buffer);
  }

  static sanitizeFilename(filename: string): string {
    // Remove dangerous characters
    return filename
      .replace(/[^a-zA-Z0-9._-]/g, '_')
      .replace(/_{2,}/g, '_')
      .toLowerCase();
  }
}

// Use in controller
@Post('upload')
@UseInterceptors(FileInterceptor('file'))
async uploadFile(@UploadedFile() file: Express.Multer.File) {
  await FileValidator.validate(file);

  const sanitizedFilename = FileValidator.sanitizeFilename(file.originalname);

  // Continue with upload
}
```

### 2. Virus Scanning with ClamAV

```typescript
import NodeClam from 'clamscan';

@Injectable()
export class AntivirusService {
  private clamscan: NodeClam;

  async onModuleInit() {
    this.clamscan = await new NodeClam().init({
      removeInfected: false,
      quarantineInfected: false,
      scanLog: null,
      debugMode: false,
      clamdscan: {
        host: 'localhost',
        port: 3310,
      },
    });
  }

  async scanFile(filePath: string): Promise<{ isInfected: boolean; viruses: string[] }> {
    const { isInfected, viruses } = await this.clamscan.isInfected(filePath);

    if (isInfected) {
      console.warn(`Infected file detected: ${filePath}`, viruses);
    }

    return { isInfected, viruses };
  }

  async scanBuffer(buffer: Buffer): Promise<{ isInfected: boolean; viruses: string[] }> {
    const tempFile = `/tmp/${Date.now()}-scan.tmp`;
    await fs.promises.writeFile(tempFile, buffer);

    try {
      return await this.scanFile(tempFile);
    } finally {
      await fs.promises.unlink(tempFile);
    }
  }
}
```

---

## Streaming Large Files

### 1. Stream Download

```typescript
@Controller('api/v1/files')
export class FileController {
  @Get(':id/download')
  async downloadFile(@Param('id') id: string, @Res() res: Response) {
    const file = await this.fileRepository.findById(new FileId(id));

    if (!file) {
      throw new NotFoundException('File not found');
    }

    // Set headers
    res.set({
      'Content-Type': file.mimeType,
      'Content-Disposition': `attachment; filename="${file.originalName}"`,
      'Content-Length': file.size,
    });

    // Stream from S3
    const stream = await this.s3Service.getFileStream(file.path);
    stream.pipe(res);
  }

  @Get(':id/stream')
  async streamFile(@Param('id') id: string, @Res() res: Response) {
    const file = await this.fileRepository.findById(new FileId(id));

    if (!file) {
      throw new NotFoundException('File not found');
    }

    res.set({
      'Content-Type': file.mimeType,
      'Content-Disposition': `inline; filename="${file.originalName}"`,
      'Accept-Ranges': 'bytes',
    });

    const stream = await this.s3Service.getFileStream(file.path);
    stream.pipe(res);
  }
}
```

### 2. Range Requests (Video/Audio Streaming)

```typescript
@Get(':id/stream')
async streamWithRange(
  @Param('id') id: string,
  @Req() req: Request,
  @Res() res: Response,
) {
  const file = await this.fileRepository.findById(new FileId(id));

  if (!file) {
    throw new NotFoundException('File not found');
  }

  const range = req.headers.range;

  if (!range) {
    // No range header, send entire file
    const stream = await this.s3Service.getFileStream(file.path);
    res.set({
      'Content-Type': file.mimeType,
      'Content-Length': file.size,
    });
    stream.pipe(res);
    return;
  }

  // Parse range header
  const parts = range.replace(/bytes=/, '').split('-');
  const start = parseInt(parts[0], 10);
  const end = parts[1] ? parseInt(parts[1], 10) : file.size - 1;
  const chunkSize = end - start + 1;

  // Get stream with range
  const stream = await this.s3Service.getFileStreamWithRange(file.path, start, end);

  res.status(206); // Partial Content
  res.set({
    'Content-Range': `bytes ${start}-${end}/${file.size}`,
    'Accept-Ranges': 'bytes',
    'Content-Length': chunkSize,
    'Content-Type': file.mimeType,
  });

  stream.pipe(res);
}
```

---

## Progress Tracking

### 1. Upload Progress (Client)

```typescript
function FileUploadWithProgress() {
  const [progress, setProgress] = useState(0);

  const uploadFile = async (file: File) => {
    const formData = new FormData();
    formData.comend('file', file);

    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener('progress', (event) => {
      if (event.lengthComputable) {
        const percentComplete = (event.loaded / event.total) * 100;
        setProgress(Math.round(percentComplete));
      }
    });

    xhr.addEventListener('load', () => {
      if (xhr.status === 200) {
        console.log('Upload complete');
        const response = JSON.parse(xhr.responseText);
        console.log(response);
      }
    });

    xhr.addEventListener('error', () => {
      console.error('Upload failed');
    });

    xhr.open('POST', 'http://localhost:4001/api/v1/files/upload');
    xhr.setRequestHeader('Authorization', `Bearer ${token}`);
    xhr.send(formData);
  };

  return (
    <div>
      <input type="file" onChange={(e) => e.target.files && uploadFile(e.target.files[0])} />
      {progress > 0 && (
        <div>
          <progress value={progress} max="100" />
          <span>{progress}%</span>
        </div>
      )}
    </div>
  );
}
```

### 2. Server-Side Progress (WebSocket)

```typescript
@WebSocketGateway()
export class FileUploadGateway {
  @SubscribeMessage('upload:start')
  async handleUploadStart(
    @MessageBody() data: { uploadId: string; filename: string; totalSize: number },
    @ConnectedSocket() client: Socket,
  ) {
    // Track upload session
  }

  notifyProgress(uploadId: string, loaded: number, total: number) {
    this.server.emit(`upload:progress:${uploadId}`, {
      loaded,
      total,
      progress: (loaded / total) * 100,
    });
  }
}
```

---

## Image Processing

### 1. Image Resizing with Sharp

```bash
pnpm add sharp
```

```typescript
import sharp from 'sharp';

@Injectable()
export class ImageProcessingService {
  async resizeImage(buffer: Buffer, width: number, height: number): Promise<Buffer> {
    return sharp(buffer)
      .resize(width, height, {
        fit: 'cover',
        position: 'center',
      })
      .jpeg({ quality: 80 })
      .toBuffer();
  }

  async createThumbnail(buffer: Buffer): Promise<Buffer> {
    return sharp(buffer)
      .resize(200, 200, {
        fit: 'cover',
      })
      .jpeg({ quality: 70 })
      .toBuffer();
  }

  async createAvatarSizes(buffer: Buffer): Promise<{
    small: Buffer;
    medium: Buffer;
    large: Buffer;
  }> {
    return {
      small: await this.resizeImage(buffer, 50, 50),
      medium: await this.resizeImage(buffer, 150, 150),
      large: await this.resizeImage(buffer, 300, 300),
    };
  }

  async optimizeImage(buffer: Buffer): Promise<Buffer> {
    return sharp(buffer).jpeg({ quality: 85, progressive: true }).toBuffer();
  }
}
```

### 2. Avatar Upload with Processing

```typescript
@Post('avatar')
@UseInterceptors(FileInterceptor('avatar'))
async uploadAvatar(
  @UploadedFile() file: Express.Multer.File,
  @CurrentUser() user: CurrentUserData,
) {
  if (!file.mimetype.startsWith('image/')) {
    throw new BadRequestException('Only image files are allowed');
  }

  // Process image
  const sizes = await this.imageService.createAvatarSizes(file.buffer);

  // Upload all sizes to S3
  const [small, medium, large] = await Promise.all([
    this.s3Service.uploadFile(
      { ...file, buffer: sizes.small },
      `avatars/${user.userId}/small.jpg`,
    ),
    this.s3Service.uploadFile(
      { ...file, buffer: sizes.medium },
      `avatars/${user.userId}/medium.jpg`,
    ),
    this.s3Service.uploadFile(
      { ...file, buffer: sizes.large },
      `avatars/${user.userId}/large.jpg`,
    ),
  ]);

  // Update user avatar URLs
  await this.commandBus.execute(
    new UpdateUserAvatarCommand(user.userId, {
      small: small.url,
      medium: medium.url,
      large: large.url,
    }),
  );

  return {
    small: small.url,
    medium: medium.url,
    large: large.url,
  };
}
```

---

## Download Patterns

### 1. Direct Download

```typescript
@Get(':id/download')
async download(
  @Param('id') id: string,
  @Res() res: Response,
) {
  const file = await this.fileRepository.findById(new FileId(id));

  if (!file) {
    throw new NotFoundException('File not found');
  }

  const stream = await this.s3Service.getFileStream(file.path);

  res.set({
    'Content-Type': file.mimeType,
    'Content-Disposition': `attachment; filename="${file.originalName}"`,
    'Content-Length': file.size,
  });

  stream.pipe(res);
}
```

### 2. Pre-Signed URL Download

```typescript
@Get(':id/download-url')
async getDownloadUrl(
  @Param('id') id: string,
  @CurrentUser() user: CurrentUserData,
) {
  const file = await this.fileRepository.findById(new FileId(id));

  if (!file) {
    throw new NotFoundException('File not found');
  }

  // Check permissions
  if (!await this.canAccessFile(user, file)) {
    throw new ForbiddenException('Access denied');
  }

  // Generate short-lived signed URL
  const url = await this.s3Service.getSignedUrl(file.path, 300); // 5 minutes

  return { url };
}
```

### 3. ZIP Archive Download

```bash
pnpm add archiver
```

```typescript
import archiver from 'archiver';

@Get('project/:projectId/download-all')
async downloadProjectFiles(
  @Param('projectId') projectId: string,
  @Res() res: Response,
) {
  const files = await this.fileRepository.findByProjectId(projectId);

  if (files.length === 0) {
    throw new NotFoundException('No files found for this project');
  }

  const archive = archiver('zip', {
    zlib: { level: 9 }, // Maximum compression
  });

  res.set({
    'Content-Type': 'application/zip',
    'Content-Disposition': `attachment; filename="project-${projectId}-files.zip"`,
  });

  archive.pipe(res);

  for (const file of files) {
    const stream = await this.s3Service.getFileStream(file.path);
    archive.comend(stream, { name: file.originalName });
  }

  await archive.finalize();
}
```

---

## Best Practices

### ✅ File Upload Checklist

- [ ] Validate file type (MIME type + magic bytes)
- [ ] Limit file size appropriately
- [ ] Sanitize filenames
- [ ] Use unique filenames (prevent collisions)
- [ ] Scan for viruses (production)
- [ ] Store metadata in database
- [ ] Use cloud storage for production (S3, etc.)
- [ ] Generate thumbnails for images
- [ ] Implement access control
- [ ] Track upload progress for large files

### ✅ Security Checklist

- [ ] Never trust client-supplied MIME types
- [ ] Verify actual file type from content
- [ ] Store files outside web root
- [ ] Use pre-signed URLs for access
- [ ] Implement rate limiting
- [ ] Audit file access
- [ ] Encrypt sensitive files at rest
- [ ] Use HTTPS for all transfers
- [ ] Implement virus scanning
- [ ] Validate file content (e.g., parse PDFs)

### ✅ Performance Checklist

- [ ] Use streaming for large files
- [ ] Implement chunked uploads for files > 10MB
- [ ] Use CDN for static file delivery
- [ ] Compress images automatically
- [ ] Generate multiple sizes for images
- [ ] Implement caching headers
- [ ] Use range requests for video/audio
- [ ] Clean up temporary files
- [ ] Monitor storage usage
- [ ] Implement file retention policies

---

## Related Patterns

- **Pattern 39**: [Security Patterns Guide](./39-Security-Patterns-Guide.md)
- **Pattern 41**: [REST API Best Practices](./41-REST-API-Best-Practices.md)
- **Pattern 45**: [Background Job Patterns](./45-Background-Job-Patterns.md)

---

## References

- [Multer Documentation](https://github.com/expressjs/multer)
- [AWS S3 SDK](https://docs.aws.amazon.com/AWSJavaScriptSDK/v3/latest/clients/client-s3/)
- [Sharp Image Processing](https://sharp.pixelplumbing.com/)
- [File Type Detection](https://github.com/sindresorhus/file-type)

---

**Last Updated**: October 8, 2025
**Version**: 1.0
**Status**: Active
