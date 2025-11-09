# Pattern 83: SCADA Protocol Adapter Pattern

**Status**: 📋 Planned (Design Complete, Implementation Pending)
**Category**: Infrastructure / Integration / SCADA
**Related Patterns**: [14 - Anti-Corruption Layer](./14-Anti-Corruption-Layer-Pattern.md), [81 - Multi-Tenant SCADA Ingestion](./81-Multi-Tenant-SCADA-Ingestion-Pattern.md)

## Problem

Oil & Gas well sites use **many different SCADA protocols** depending on:
- Equipment age (legacy vs modern)
- Vendor (Emerson, Schneider, Rockwell, etc.)
- Geographic region (North America vs Europe vs Middle East)
- Communication infrastructure (serial, Ethernet, cellular)

**Common protocols in the field:**
- **OPC-UA**: Modern PLCs and DCS systems (Emerson DeltaV, Schneider Wonderware)
- **Modbus RTU/TCP**: Legacy RTUs and older equipment (extremely common)
- **MQTT**: IoT sensors and edge devices
- **DNP3**: SCADA systems and utilities
- **HART**: Smart field instrumentation (pressure transmitters, flow meters)
- **Profibus/Profinet**: Siemens ecosystems
- **Foundation Fieldbus**: Process control networks

**The Challenge**: How do you build a SCADA ingestion service that supports **all these protocols** without creating protocol-specific services?

**Anti-Patterns to Avoid:**
- ❌ Building separate ingestion services per protocol (Modbus service, OPC-UA service, etc.)
- ❌ Hardcoding protocol logic into the core ingestion service
- ❌ Tight coupling between protocol implementation and data storage
- ❌ Requiring code changes to add new protocols

## Context

WellOS needs to ingest production data from **thousands of well sites** across multiple operators, each with different equipment and protocols:

```
Operator A (Modern)          Operator B (Mixed)           Operator C (Legacy)
├─ OPC-UA (90% of wells)    ├─ OPC-UA (50%)             ├─ Modbus RTU (80%)
├─ MQTT (IoT sensors)       ├─ Modbus TCP (30%)         ├─ DNP3 (15%)
└─ Modbus TCP (legacy)      ├─ MQTT (IoT)               └─ HART (5%)
                            └─ HART (instruments)
```

**Key Requirements:**
- Support multiple protocols **simultaneously** (same tenant may have different wells using different protocols)
- Add new protocols **without changing core service**
- Translate all protocols to a **common internal format**
- Maintain tenant isolation across all protocols
- Abstract away protocol complexity from the aggregation/storage layer

## Solution

### Protocol Adapter Pattern

Create a **pluggable adapter layer** where:
1. Each protocol gets its own adapter (implements a common trait/interface)
2. All adapters translate to a common internal format (`ProtocolReading`)
3. Core service is protocol-agnostic (doesn't know about specific protocols)
4. New protocols are added by implementing the adapter trait

### Architecture

```rust
// 1. Common internal format (protocol-agnostic)
pub struct ProtocolReading {
    pub timestamp: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub well_id: Uuid,
    pub tag_name: String,        // e.g., "oil_rate", "tubing_pressure"
    pub value: f64,
    pub quality: ReadingQuality, // Good, Bad, Uncertain
    pub source_protocol: String, // "OPC-UA", "Modbus", "MQTT", etc. (for debugging)
}

pub enum ReadingQuality {
    Good,
    Bad,
    Uncertain,
}

// 2. Protocol Adapter Trait (Rust)
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Connect to the remote device
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), ProtocolError>;

    /// Subscribe to tags (for protocols that support subscriptions like OPC-UA, MQTT)
    async fn subscribe(&mut self, tags: Vec<TagMapping>) -> Result<(), ProtocolError>;

    /// Poll for readings (for protocols that require polling like Modbus, DNP3)
    async fn poll(&mut self) -> Result<Vec<ProtocolReading>, ProtocolError>;

    /// Disconnect gracefully
    async fn disconnect(&mut self) -> Result<(), ProtocolError>;

    /// Get protocol name (for logging/debugging)
    fn protocol_name(&self) -> &str;
}

// 3. Adapter Factory (creates adapters based on protocol type)
pub struct AdapterFactory;

impl AdapterFactory {
    pub fn create_adapter(protocol: &str) -> Result<Box<dyn ProtocolAdapter>, ProtocolError> {
        match protocol {
            "OPC-UA" => Ok(Box::new(OpcUaAdapter::new())),
            "Modbus-TCP" => Ok(Box::new(ModbusTcpAdapter::new())),
            "Modbus-RTU" => Ok(Box::new(ModbusRtuAdapter::new())),
            "MQTT" => Ok(Box::new(MqttAdapter::new())),
            "DNP3" => Ok(Box::new(Dnp3Adapter::new())),
            "HART-IP" => Ok(Box::new(HartIpAdapter::new())),
            _ => Err(ProtocolError::UnsupportedProtocol(protocol.to_string())),
        }
    }
}
```

### Example Adapters

#### OPC-UA Adapter
```rust
use opcua::client::prelude::*;

pub struct OpcUaAdapter {
    client: Option<Client>,
    session: Option<Session>,
    subscription_handle: Option<u32>,
}

#[async_trait]
impl ProtocolAdapter for OpcUaAdapter {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), ProtocolError> {
        // Parse OPC-UA specific config
        let endpoint_url = &config.endpoint_url;
        let security_mode = config.security_mode.as_deref().unwrap_or("None");

        // Create OPC-UA client
        let client = ClientBuilder::new()
            .comlication_name("WellOS SCADA Ingestion")
            .comlication_uri("urn:WellOS:SCADA")
            .create_sample_keypair(true)
            .trust_server_certs(true)
            .session_retry_limit(3)
            .client()?;

        // Connect to server
        let session = client.connect_to_endpoint(
            (endpoint_url, SecurityPolicy::from_str(security_mode)?),
            IdentityToken::Anonymous,
        ).await?;

        self.client = Some(client);
        self.session = Some(session);

        Ok(())
    }

    async fn subscribe(&mut self, tags: Vec<TagMapping>) -> Result<(), ProtocolError> {
        let session = self.session.as_mut()
            .ok_or(ProtocolError::NotConnected)?;

        // Create subscription
        let subscription_id = session.create_subscription(
            500.0,  // Publishing interval (ms)
            10,     // Lifetime count
            3,      // Max keep-alive count
            0,      // Max notifications per publish
            true,   // Publishing enabled
            0,      // Priority
        ).await?;

        // Create monitored items for each tag
        let mut items_to_create = Vec::new();
        for tag in tags {
            items_to_create.push(MonitoredItemCreateRequest {
                item_to_monitor: ReadValueId {
                    node_id: NodeId::from_str(&tag.opc_node_id)?,
                    attribute_id: AttributeId::Value as u32,
                    ..Default::default()
                },
                monitoring_mode: MonitoringMode::Reporting,
                requested_parameters: MonitoringParameters {
                    client_handle: tag.id.as_u128() as u32,
                    sampling_interval: 1000.0, // 1 second
                    queue_size: 1,
                    discard_oldest: true,
                    ..Default::default()
                },
            });
        }

        session.create_monitored_items(subscription_id, items_to_create).await?;

        self.subscription_handle = Some(subscription_id);

        Ok(())
    }

    async fn poll(&mut self) -> Result<Vec<ProtocolReading>, ProtocolError> {
        let session = self.session.as_mut()
            .ok_or(ProtocolError::NotConnected)?;

        // OPC-UA is subscription-based, so poll just retrieves notifications
        let notifications = session.receive_publish_responses().await?;

        let mut readings = Vec::new();
        for notification in notifications {
            if let Some(data_value) = notification.value {
                readings.push(ProtocolReading {
                    timestamp: data_value.server_timestamp.unwrap_or_else(Utc::now),
                    tenant_id: notification.tenant_id,
                    well_id: notification.well_id,
                    tag_name: notification.tag_name,
                    value: data_value.value.as_f64()?,
                    quality: match data_value.status {
                        StatusCode::Good => ReadingQuality::Good,
                        StatusCode::Bad => ReadingQuality::Bad,
                        _ => ReadingQuality::Uncertain,
                    },
                    source_protocol: "OPC-UA".to_string(),
                });
            }
        }

        Ok(readings)
    }

    async fn disconnect(&mut self) -> Result<(), ProtocolError> {
        if let Some(session) = self.session.take() {
            session.disconnect().await?;
        }
        Ok(())
    }

    fn protocol_name(&self) -> &str {
        "OPC-UA"
    }
}
```

#### Modbus TCP Adapter
```rust
use tokio_modbus::prelude::*;

pub struct ModbusTcpAdapter {
    context: Option<client::Context>,
    tags: Vec<TagMapping>,
}

#[async_trait]
impl ProtocolAdapter for ModbusTcpAdapter {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), ProtocolError> {
        // Parse Modbus TCP config
        let socket_addr = config.endpoint_url.parse::<SocketAddr>()?;

        // Connect to Modbus TCP device
        let context = tcp::connect(socket_addr).await?;

        self.context = Some(context);

        Ok(())
    }

    async fn subscribe(&mut self, tags: Vec<TagMapping>) -> Result<(), ProtocolError> {
        // Modbus doesn't support subscriptions - store tags for polling
        self.tags = tags;
        Ok(())
    }

    async fn poll(&mut self) -> Result<Vec<ProtocolReading>, ProtocolError> {
        let context = self.context.as_mut()
            .ok_or(ProtocolError::NotConnected)?;

        let mut readings = Vec::new();
        let now = Utc::now();

        // Poll each tag
        for tag in &self.tags {
            // Parse Modbus address from opc_node_id (e.g., "40001" for holding register 1)
            let register_address = tag.opc_node_id.parse::<u16>()
                .map_err(|_| ProtocolError::InvalidAddress)?;

            // Determine register type (coil, discrete input, input register, holding register)
            let register_type = if register_address < 10000 {
                "coil"
            } else if register_address < 20000 {
                "discrete_input"
            } else if register_address < 40000 {
                "input_register"
            } else {
                "holding_register"
            };

            // Read appropriate register type
            let value = match register_type {
                "coil" => {
                    let coils = context.read_coils(register_address, 1).await?;
                    if coils[0] { 1.0 } else { 0.0 }
                }
                "discrete_input" => {
                    let inputs = context.read_discrete_inputs(register_address, 1).await?;
                    if inputs[0] { 1.0 } else { 0.0 }
                }
                "input_register" => {
                    let registers = context.read_input_registers(register_address, 1).await?;
                    registers[0] as f64
                }
                "holding_register" => {
                    let registers = context.read_holding_registers(register_address - 40000, 1).await?;
                    registers[0] as f64
                }
                _ => return Err(ProtocolError::InvalidAddress),
            };

            readings.push(ProtocolReading {
                timestamp: now,
                tenant_id: tag.tenant_id,
                well_id: tag.well_id,
                tag_name: tag.tag_name.clone(),
                value,
                quality: ReadingQuality::Good, // Modbus doesn't report quality
                source_protocol: "Modbus-TCP".to_string(),
            });
        }

        Ok(readings)
    }

    async fn disconnect(&mut self) -> Result<(), ProtocolError> {
        // Modbus TCP doesn't require explicit disconnect
        self.context = None;
        Ok(())
    }

    fn protocol_name(&self) -> &str {
        "Modbus-TCP"
    }
}
```

#### MQTT Adapter
```rust
use rumqttc::{AsyncClient, MqttOptions, QoS};

pub struct MqttAdapter {
    client: Option<AsyncClient>,
    topics: Vec<String>,
}

#[async_trait]
impl ProtocolAdapter for MqttAdapter {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), ProtocolError> {
        // Parse MQTT config (e.g., "mqtt://broker.example.com:1883")
        let url = Url::parse(&config.endpoint_url)?;
        let host = url.host_str().ok_or(ProtocolError::InvalidUrl)?;
        let port = url.port().unwrap_or(1883);

        // Create MQTT client
        let mut mqtt_options = MqttOptions::new("wellos-scada", host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        // Authentication (if configured)
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        // Start event loop in background
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        // Process MQTT events
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {}", e);
                        break;
                    }
                }
            }
        });

        self.client = Some(client);

        Ok(())
    }

    async fn subscribe(&mut self, tags: Vec<TagMapping>) -> Result<(), ProtocolError> {
        let client = self.client.as_ref()
            .ok_or(ProtocolError::NotConnected)?;

        // Subscribe to MQTT topics for each tag
        for tag in tags {
            // Tag mapping: opc_node_id contains MQTT topic (e.g., "well/123/oil_rate")
            let topic = tag.opc_node_id.clone();
            client.subscribe(&topic, QoS::AtLeastOnce).await?;
            self.topics.push(topic);
        }

        Ok(())
    }

    async fn poll(&mut self) -> Result<Vec<ProtocolReading>, ProtocolError> {
        // MQTT is subscription-based - readings come via event loop callbacks
        // This would typically be implemented with a channel that receives messages
        // For brevity, simplified here
        Ok(vec![])
    }

    async fn disconnect(&mut self) -> Result<(), ProtocolError> {
        if let Some(client) = self.client.take() {
            client.disconnect().await?;
        }
        Ok(())
    }

    fn protocol_name(&self) -> &str {
        "MQTT"
    }
}
```

### Database Schema (Connection Configuration)

The `scada_connections` table needs a `protocol_type` field:

```sql
CREATE TABLE scada_connections (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    well_id uuid NOT NULL REFERENCES wells(id),

    -- Protocol configuration
    protocol_type varchar(50) NOT NULL,  -- "OPC-UA", "Modbus-TCP", "MQTT", etc.
    endpoint_url varchar(500) NOT NULL,

    -- Security (protocol-specific)
    security_mode varchar(50),           -- "None", "Sign", "SignAndEncrypt" (OPC-UA)
    security_policy varchar(50),         -- "None", "Basic256Sha256" (OPC-UA)
    username varchar(255),               -- For authentication
    password_encrypted text,             -- Encrypted credentials

    -- Modbus-specific
    slave_id integer,                    -- Modbus slave/unit ID

    -- MQTT-specific
    client_id varchar(255),              -- MQTT client ID

    -- Status
    is_enabled boolean NOT NULL DEFAULT true,
    last_connected_at timestamptz,
    last_error text,

    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
```

### Core Service (Protocol-Agnostic)

```rust
pub struct ScadaIngestionService {
    adapters: Arc<RwLock<HashMap<Uuid, Box<dyn ProtocolAdapter>>>>,
    aggregator: Arc<Aggregator>,
}

impl ScadaIngestionService {
    pub async fn start(&self) -> Result<(), IngestionError> {
        // Load connections from all tenant databases
        let connections = self.load_all_connections().await?;

        // Create adapters for each connection
        for conn in connections {
            // Create appropriate adapter based on protocol_type
            let mut adapter = AdapterFactory::create_adapter(&conn.protocol_type)?;

            // Connect to device
            adapter.connect(&conn.config).await?;

            // Subscribe to tags
            let tags = self.load_tag_mappings(conn.tenant_id, conn.id).await?;
            adapter.subscribe(tags).await?;

            // Store adapter
            self.adapters.write().await.insert(conn.id, adapter);

            info!("Started {} connection for well {}", conn.protocol_type, conn.well_id);
        }

        // Start polling loop
        self.poll_all_adapters().await?;

        Ok(())
    }

    async fn poll_all_adapters(&self) -> Result<(), IngestionError> {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let adapters = self.adapters.read().await;

            for (conn_id, adapter) in adapters.iter() {
                // Poll adapter for new readings
                match adapter.poll().await {
                    Ok(readings) => {
                        // Send readings to aggregator (protocol-agnostic from here on)
                        for reading in readings {
                            self.aggregator.add_reading(reading).await;
                        }
                    }
                    Err(e) => {
                        error!("Error polling adapter {}: {}", conn_id, e);
                    }
                }
            }
        }
    }
}
```

## Benefits

### 1. Protocol Independence
- Core service doesn't know about specific protocols
- Add new protocols without changing core logic
- Each protocol can have its own implementation quirks handled in its adapter

### 2. Flexibility
- **Same tenant can use multiple protocols** (legacy Modbus + modern OPC-UA)
- **Easy to add new protocols** (just implement the trait)
- **Protocol-specific optimizations** (Modbus polling vs OPC-UA subscriptions)

### 3. Maintainability
- **Separation of concerns**: Protocol complexity isolated in adapters
- **Easy to test**: Mock the adapter trait for unit tests
- **Clear contracts**: Trait defines what all adapters must implement

### 4. Scalability
- **Adapter instances are independent** (one failing adapter doesn't affect others)
- **Parallel polling**: Each adapter can poll concurrently
- **Resource isolation**: Adapters can have different connection pools

## Trade-offs

### ✅ Advantages
- **Protocol-agnostic core**: Easy to add new protocols
- **Tenant flexibility**: Different tenants can use different protocols
- **Isolation**: Protocol issues don't affect the whole service
- **Testability**: Mock adapters for testing

### ❌ Disadvantages
- **More code**: Each protocol needs an adapter implementation
- **Learning curve**: Developers need to understand each protocol
- **Dependency bloat**: More Rust crates for each protocol library
- **Protocol-specific bugs**: Each adapter can have its own issues

## Implementation Priority

**Phase 1: Core Protocols (80% coverage)**
1. ✅ **OPC-UA** - Already implemented (Pattern 81)
2. 🔨 **Modbus TCP** - Most common legacy protocol
3. 🔨 **MQTT** - IoT sensors and edge devices

**Phase 2: Extended Support (95% coverage)**
4. 📋 **Modbus RTU** - Serial Modbus (requires serial port support)
5. 📋 **DNP3** - Utility-grade SCADA
6. 📋 **HART-IP** - Smart instrumentation (Ethernet-enabled)

**Phase 3: Specialized (98% coverage)**
7. 📋 **Profibus/Profinet** - Siemens ecosystems
8. 📋 **BACnet** - Building automation (if expanding beyond O&G)
9. 📋 **Custom REST APIs** - Vendor-specific cloud platforms

## Real-World Usage

### Multi-Protocol Tenant

```sql
-- Operator has mixed infrastructure
INSERT INTO scada_connections (tenant_id, well_id, protocol_type, endpoint_url) VALUES
-- Modern wells with OPC-UA
('tenant-123', 'well-1', 'OPC-UA', 'opc.tcp://192.168.1.100:4840'),
('tenant-123', 'well-2', 'OPC-UA', 'opc.tcp://192.168.1.101:4840'),

-- Legacy wells with Modbus TCP
('tenant-123', 'well-3', 'Modbus-TCP', '192.168.1.200:502'),
('tenant-123', 'well-4', 'Modbus-TCP', '192.168.1.201:502'),

-- IoT sensors with MQTT
('tenant-123', 'well-5', 'MQTT', 'mqtt://mqtt.tenant-123.com:1883');
```

### Service Startup

```
[INFO] Starting SCADA Ingestion Service v0.2.0
[INFO] Loaded 5 connections for tenant tenant-123
[INFO] Creating OPC-UA adapter for well-1
[INFO] Creating OPC-UA adapter for well-2
[INFO] Creating Modbus-TCP adapter for well-3
[INFO] Creating Modbus-TCP adapter for well-4
[INFO] Creating MQTT adapter for well-5
[INFO] All adapters started successfully
[INFO] Polling loop started (5-second interval)
```

## Testing

### Mock Adapter for Tests

```rust
pub struct MockAdapter {
    pub readings: Vec<ProtocolReading>,
}

#[async_trait]
impl ProtocolAdapter for MockAdapter {
    async fn connect(&mut self, _config: &ConnectionConfig) -> Result<(), ProtocolError> {
        Ok(())
    }

    async fn subscribe(&mut self, _tags: Vec<TagMapping>) -> Result<(), ProtocolError> {
        Ok(())
    }

    async fn poll(&mut self) -> Result<Vec<ProtocolReading>, ProtocolError> {
        Ok(self.readings.clone())
    }

    async fn disconnect(&mut self) -> Result<(), ProtocolError> {
        Ok(())
    }

    fn protocol_name(&self) -> &str {
        "Mock"
    }
}

#[tokio::test]
async fn test_ingestion_service_with_mock_adapter() {
    let mut mock_adapter = MockAdapter {
        readings: vec![
            ProtocolReading {
                timestamp: Utc::now(),
                tenant_id: Uuid::new_v4(),
                well_id: Uuid::new_v4(),
                tag_name: "oil_rate".to_string(),
                value: 250.0,
                quality: ReadingQuality::Good,
                source_protocol: "Mock".to_string(),
            },
        ],
    };

    let readings = mock_adapter.poll().await.unwrap();
    assert_eq!(readings.len(), 1);
    assert_eq!(readings[0].value, 250.0);
}
```

## Related Patterns

- **[Pattern 14: Anti-Corruption Layer](./14-Anti-Corruption-Layer-Pattern.md)** - External system integration abstraction
- **[Pattern 81: Multi-Tenant SCADA Ingestion](./81-Multi-Tenant-SCADA-Ingestion-Pattern.md)** - Core ingestion service architecture
- **[Pattern 10: Strategy Pattern](./10-Strategy-Pattern.md)** - Algorithm encapsulation (similar to adapter pattern)

## References

- WellOS Implementation: `apps/scada-ingestion/src/adapters/`
- OPC-UA Rust Client: https://github.com/locka99/opcua
- Tokio Modbus: https://github.com/slowtec/tokio-modbus
- MQTT Rust Client: https://github.com/bytebeamio/rumqtt
- Industrial Protocol Standards: https://www.isa.org/standards-and-publications

## Version History

- **v1.0** (2025-10-30): Initial pattern documentation with OPC-UA, Modbus TCP, and MQTT adapters

---

*Pattern ID: 83*
*Created: 2025-10-30*
*Last Updated: 2025-10-30*
