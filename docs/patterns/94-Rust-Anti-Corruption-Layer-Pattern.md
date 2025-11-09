# Pattern 94: Rust Anti-Corruption Layer Pattern for SCADA Integration

**Version**: 1.0
**Last Updated**: November 3, 2025
**Category**: Integration & External Systems
**Complexity**: Advanced
**Status**: Recommended

---

## Table of Contents

1. [Overview](#overview)
2. [Problem](#problem)
3. [Solution](#solution)
4. [WellOS Use Cases](#wellos-use-cases)
5. [Rust Implementation](#rust-implementation)
6. [Protocol-Specific Adapters](#protocol-specific-adapters)
7. [Error Handling](#error-handling)
8. [Testing](#testing)
9. [Benefits](#benefits)
10. [Trade-offs](#trade-offs)
11. [Related Patterns](#related-patterns)

---

## Overview

The **Anti-Corruption Layer (ACL) Pattern** creates a translation boundary between WellOS's domain model and external systems with incompatible data structures. In the context of WellOS's Rust-based SCADA ingestion service, this pattern isolates the clean domain model from the messy reality of industrial SCADA protocols (Modbus, OPC UA, DNP3, MQTT, etc.).

**Core Principle**: External systems should never pollute your domain model.

```
External System (Modbus RTU)  →  Anti-Corruption Layer  →  WellOS Domain Model
  - Register addresses              - Protocol translation      - Clean sensor readings
  - Binary data formats             - Unit conversions          - Typed measurements
  - Protocol-specific errors        - Domain error types        - Business rules
```

---

## Problem

Integrating with external SCADA systems creates several challenges:

### 1. Incompatible Data Models

```rust
// ❌ External Modbus system exposes raw register data
struct ModbusResponse {
    function_code: u8,
    register_address: u16,
    register_value: u16,  // Raw 16-bit integer
    byte_count: u8,
}

// ✅ WellOS domain expects typed sensor data
struct SensorReading {
    well_id: WellId,
    sensor_type: SensorType,
    value: f64,              // Converted to engineering units
    unit: MeasurementUnit,   // PSI, BPD, SCFM, etc.
    timestamp: DateTime<Utc>,
    quality: DataQuality,
}
```

### 2. Protocol-Specific Error Handling

```rust
// ❌ Modbus-specific error codes leak into domain
enum ModbusError {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    ServerDeviceFailure = 0x04,
}

// ✅ Domain-level error types
enum SensorReadError {
    SensorUnavailable { sensor_id: String },
    InvalidReading { reason: String },
    CommunicationFailure,
}
```

### 3. Vendor Lock-In

```rust
// ❌ Direct dependency on Modbus throughout codebase
impl WellMonitor {
    async fn read_pressure(&self) -> ModbusResult<u16> {
        self.modbus_client.read_holding_register(0x0001).await
    }
}

// ✅ Abstraction over protocol details
impl WellMonitor {
    async fn read_pressure(&self) -> Result<Pressure> {
        self.sensor_gateway.read_sensor(SensorId::Pressure).await
    }
}
```

### 4. Multiple SCADA Protocols

WellOS must support 8+ industrial protocols:

- **Modbus TCP/RTU** - Legacy PLCs, RTUs
- **OPC UA** - Modern SCADA systems
- **DNP3** - Utility SCADA
- **MQTT** - IoT sensors
- **EtherNet/IP** - Allen-Bradley PLCs
- **HART-IP** - Smart field devices
- **BACnet** - Building automation systems
- **Proprietary APIs** - Vendor-specific systems

Each protocol has different:
- Data encoding (big-endian vs little-endian)
- Authentication mechanisms
- Connection management
- Error handling
- Retry strategies

---

## Solution

Create protocol-specific **Anti-Corruption Layers** that translate between external SCADA protocols and WellOS's domain model.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WellOS Core Domain                     │
│  (Protocol-agnostic sensor readings, well data, alarms)     │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
┌───────────────────────────┴─────────────────────────────────┐
│              Sensor Gateway (Protocol Abstraction)          │
│  trait SensorProtocol { async fn read_sensor(...) }         │
└─────────────────────────────────────────────────────────────┘
                            ▲
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────┴────────┐  ┌───────┴────────┐  ┌──────┴─────────┐
│  Modbus ACL    │  │   OPC UA ACL   │  │   DNP3 ACL     │
│  - Register    │  │  - Node IDs    │  │  - Point IDs   │
│    mapping     │  │  - Data types  │  │  - Quality     │
│  - Scaling     │  │  - Security    │  │    flags       │
└───────┬────────┘  └───────┬────────┘  └──────┬─────────┘
        │                   │                   │
┌───────▼────────┐  ┌───────▼────────┐  ┌──────▼─────────┐
│  Modbus Client │  │ OPC UA Client  │  │  DNP3 Client   │
│  (External)    │  │  (External)    │  │  (External)    │
└────────────────┘  └────────────────┘  └────────────────┘
```

---

## WellOS Use Cases

### 1. SCADA Protocol Adapters (Sprint 3-5)

**Problem**: Ingest sensor data from Modbus RTU controllers on remote well pads.

**ACL Solution**: Modbus register addresses and raw data formats are translated to typed sensor readings.

```rust
// External Modbus configuration (vendor-specific)
const PRESSURE_REGISTER: u16 = 0x0001;  // Casing pressure (raw)
const FLOW_REGISTER: u16 = 0x0010;      // Gas flow rate (raw)
const TEMPERATURE_REGISTER: u16 = 0x0020; // Wellhead temp (raw)

// WellOS domain model (protocol-agnostic)
enum SensorType {
    CasingPressure,
    GasFlowRate,
    WellheadTemperature,
}
```

### 2. EIA Commodity Pricing API (Sprint 5)

**Problem**: Fetch oil/gas pricing from EIA API which uses different commodity codes and date formats.

**ACL Solution**: Translate EIA API responses to WellOS's pricing domain model.

```rust
// External EIA API response
{
    "series": [{
        "series_id": "PET.RWTC.D",  // WTI Crude Oil
        "data": [["2025-11-01", "78.50"]]
    }]
}

// WellOS domain
struct CommodityPrice {
    commodity: CommodityType::OilWTI,
    price_per_unit: Decimal::new(7850, 2),  // $78.50
    unit: PriceUnit::PerBarrel,
    date: NaiveDate::from_ymd(2025, 11, 1),
}
```

### 3. QuickBooks Integration (Sprint 5)

**Problem**: Sync production revenue to QuickBooks which uses different accounting categories.

**ACL Solution**: Map WellOS production data to QuickBooks journal entries.

```rust
// WellOS domain
struct ProductionRevenue {
    well_id: WellId,
    oil_volume: f64,      // BBL
    gas_volume: f64,      // MCF
    revenue: Decimal,     // USD
}

// QuickBooks API (external)
struct JournalEntry {
    line_items: vec![
        { account: "4000-Oil Sales", debit: 15000.00 },
        { account: "4100-Gas Sales", debit: 3500.00 },
    ]
}
```

### 4. SharePoint Document Integration (Sprint 5)

**Problem**: Store production reports in client's SharePoint with metadata mapping.

**ACL Solution**: Convert WellOS report metadata to SharePoint column values.

---

## Rust Implementation

### Core Trait: Protocol Adapter

```rust
// apps/scada-ingestion/src/adapters/mod.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::error::Error;

/// Domain model: Clean sensor reading
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub well_id: String,
    pub sensor_type: SensorType,
    pub value: f64,
    pub unit: MeasurementUnit,
    pub timestamp: DateTime<Utc>,
    pub quality: DataQuality,
}

#[derive(Debug, Clone)]
pub enum SensorType {
    CasingPressure,
    TubingPressure,
    GasFlowRate,
    OilFlowRate,
    WellheadTemperature,
    MotorCurrent,
    MotorVibration,
}

#[derive(Debug, Clone)]
pub enum MeasurementUnit {
    PSI,        // Pressure
    BPD,        // Barrels per day (oil)
    SCFM,       // Standard cubic feet per minute (gas)
    Fahrenheit,
    Amperes,
    Hertz,
}

#[derive(Debug, Clone)]
pub enum DataQuality {
    Good,
    Uncertain,
    Bad,
}

/// Anti-Corruption Layer trait
/// Each SCADA protocol implements this to translate to domain model
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Protocol name (for logging/monitoring)
    fn protocol_name(&self) -> &'static str;

    /// Connect to external SCADA system
    async fn connect(&mut self) -> Result<(), Box<dyn Error>>;

    /// Read sensor value and translate to domain model
    async fn read_sensor(
        &self,
        sensor_id: &str,
    ) -> Result<SensorReading, Box<dyn Error>>;

    /// Read multiple sensors (batch operation)
    async fn read_sensors(
        &self,
        sensor_ids: &[String],
    ) -> Result<Vec<SensorReading>, Box<dyn Error>>;

    /// Disconnect from external system
    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>>;

    /// Health check
    async fn health_check(&self) -> Result<bool, Box<dyn Error>>;
}
```

### Modbus TCP Anti-Corruption Layer

```rust
// apps/scada-ingestion/src/adapters/modbus_tcp.rs
use super::{ProtocolAdapter, SensorReading, SensorType, MeasurementUnit, DataQuality};
use async_trait::async_trait;
use tokio_modbus::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use chrono::Utc;

/// Modbus register configuration (external system knowledge)
#[derive(Debug, Clone)]
struct ModbusRegisterConfig {
    address: u16,
    sensor_type: SensorType,
    scale_factor: f64,    // Convert raw value to engineering units
    unit: MeasurementUnit,
}

pub struct ModbusTcpAdapter {
    host: String,
    port: u16,
    client: Option<tokio_modbus::client::Context>,
    /// Maps sensor IDs to Modbus register configurations
    /// This encapsulates all vendor-specific knowledge
    register_map: HashMap<String, ModbusRegisterConfig>,
}

impl ModbusTcpAdapter {
    pub fn new(host: String, port: u16) -> Self {
        let mut register_map = HashMap::new();

        // Vendor-specific register mapping (ACL configuration)
        register_map.insert(
            "well-123-pressure-casing".to_string(),
            ModbusRegisterConfig {
                address: 0x0001,
                sensor_type: SensorType::CasingPressure,
                scale_factor: 0.1,  // Raw value * 0.1 = PSI
                unit: MeasurementUnit::PSI,
            },
        );

        register_map.insert(
            "well-123-flow-gas".to_string(),
            ModbusRegisterConfig {
                address: 0x0010,
                sensor_type: SensorType::GasFlowRate,
                scale_factor: 1.0,  // Direct SCFM reading
                unit: MeasurementUnit::SCFM,
            },
        );

        register_map.insert(
            "well-123-temp-wellos".to_string(),
            ModbusRegisterConfig {
                address: 0x0020,
                sensor_type: SensorType::WellheadTemperature,
                scale_factor: 0.1,  // Raw value * 0.1 = °F
                unit: MeasurementUnit::Fahrenheit,
            },
        );

        Self {
            host,
            port,
            client: None,
            register_map,
        }
    }

    /// Translate raw Modbus register value to domain sensor reading
    fn translate_register_to_reading(
        &self,
        sensor_id: &str,
        raw_value: u16,
    ) -> Result<SensorReading, Box<dyn Error>> {
        let config = self.register_map.get(sensor_id)
            .ok_or_else(|| format!("Unknown sensor ID: {}", sensor_id))?;

        // Convert raw 16-bit integer to engineering units
        let value = (raw_value as f64) * config.scale_factor;

        // Determine data quality based on raw value
        let quality = self.assess_data_quality(raw_value, &config.sensor_type);

        Ok(SensorReading {
            well_id: self.extract_well_id(sensor_id),
            sensor_type: config.sensor_type.clone(),
            value,
            unit: config.unit.clone(),
            timestamp: Utc::now(),
            quality,
        })
    }

    fn extract_well_id(&self, sensor_id: &str) -> String {
        // Extract "well-123" from "well-123-pressure-casing"
        sensor_id.split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-")
    }

    fn assess_data_quality(&self, raw_value: u16, sensor_type: &SensorType) -> DataQuality {
        // Domain business rule: Detect sensor failures
        match sensor_type {
            SensorType::CasingPressure => {
                if raw_value == 0 || raw_value == 0xFFFF {
                    DataQuality::Bad  // Sensor disconnected
                } else if raw_value > 5000 {
                    DataQuality::Uncertain  // Abnormally high reading
                } else {
                    DataQuality::Good
                }
            }
            SensorType::GasFlowRate => {
                if raw_value == 0xFFFF {
                    DataQuality::Bad
                } else {
                    DataQuality::Good
                }
            }
            _ => DataQuality::Good,
        }
    }
}

#[async_trait]
impl ProtocolAdapter for ModbusTcpAdapter {
    fn protocol_name(&self) -> &'static str {
        "Modbus TCP"
    }

    async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let socket_addr = format!("{}:{}", self.host, self.port);
        let ctx = tcp::connect(socket_addr.parse()?).await?;
        self.client = Some(ctx);
        Ok(())
    }

    async fn read_sensor(
        &self,
        sensor_id: &str,
    ) -> Result<SensorReading, Box<dyn Error>> {
        let client = self.client.as_ref()
            .ok_or("Not connected to Modbus server")?;

        let config = self.register_map.get(sensor_id)
            .ok_or_else(|| format!("Unknown sensor ID: {}", sensor_id))?;

        // Read raw Modbus register (external system interaction)
        let raw_values = client.read_holding_registers(config.address, 1).await?;
        let raw_value = raw_values.get(0)
            .ok_or("No data returned from Modbus server")?;

        // Translate to domain model (ACL transformation)
        self.translate_register_to_reading(sensor_id, *raw_value)
    }

    async fn read_sensors(
        &self,
        sensor_ids: &[String],
    ) -> Result<Vec<SensorReading>, Box<dyn Error>> {
        let mut readings = Vec::new();
        for sensor_id in sensor_ids {
            let reading = self.read_sensor(sensor_id).await?;
            readings.push(reading);
        }
        Ok(readings)
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.client = None;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn Error>> {
        if self.client.is_none() {
            return Ok(false);
        }

        // Try to read a known register
        if let Some(first_sensor) = self.register_map.keys().next() {
            match self.read_sensor(first_sensor).await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}
```

---

## Protocol-Specific Adapters

### OPC UA Anti-Corruption Layer

```rust
// apps/scada-ingestion/src/adapters/opcua.rs
use super::{ProtocolAdapter, SensorReading, SensorType, MeasurementUnit, DataQuality};
use async_trait::async_trait;
use opcua::client::prelude::*;
use std::collections::HashMap;

/// OPC UA Node ID configuration (external system knowledge)
#[derive(Debug, Clone)]
struct OpcUaNodeConfig {
    node_id: String,  // e.g., "ns=2;s=Well123.CasingPressure"
    sensor_type: SensorType,
    unit: MeasurementUnit,
}

pub struct OpcUaAdapter {
    endpoint_url: String,
    client: Option<Client>,
    session: Option<Session>,
    node_map: HashMap<String, OpcUaNodeConfig>,
}

impl OpcUaAdapter {
    pub fn new(endpoint_url: String) -> Self {
        let mut node_map = HashMap::new();

        // OPC UA node mapping (vendor-specific)
        node_map.insert(
            "well-123-pressure-casing".to_string(),
            OpcUaNodeConfig {
                node_id: "ns=2;s=Well123.Sensors.CasingPressure".to_string(),
                sensor_type: SensorType::CasingPressure,
                unit: MeasurementUnit::PSI,
            },
        );

        Self {
            endpoint_url,
            client: None,
            session: None,
            node_map,
        }
    }

    fn translate_opcua_value_to_reading(
        &self,
        sensor_id: &str,
        data_value: &DataValue,
    ) -> Result<SensorReading, Box<dyn Error>> {
        let config = self.node_map.get(sensor_id)
            .ok_or_else(|| format!("Unknown sensor ID: {}", sensor_id))?;

        // Extract value from OPC UA DataValue
        let value = match &data_value.value {
            Some(Variant::Double(v)) => *v,
            Some(Variant::Float(v)) => *v as f64,
            Some(Variant::Int32(v)) => *v as f64,
            _ => return Err("Unsupported OPC UA data type".into()),
        };

        // Translate OPC UA quality to domain quality
        let quality = match data_value.status {
            Some(StatusCode::Good) => DataQuality::Good,
            Some(StatusCode::Uncertain) => DataQuality::Uncertain,
            _ => DataQuality::Bad,
        };

        Ok(SensorReading {
            well_id: self.extract_well_id(sensor_id),
            sensor_type: config.sensor_type.clone(),
            value,
            unit: config.unit.clone(),
            timestamp: data_value.server_timestamp
                .map(|ts| DateTime::from_timestamp(ts.ticks / 10_000_000, 0).unwrap())
                .unwrap_or_else(Utc::now),
            quality,
        })
    }

    fn extract_well_id(&self, sensor_id: &str) -> String {
        sensor_id.split('-').take(2).collect::<Vec<_>>().join("-")
    }
}

#[async_trait]
impl ProtocolAdapter for OpcUaAdapter {
    fn protocol_name(&self) -> &'static str {
        "OPC UA"
    }

    async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let client = ClientBuilder::new()
            .application_name("WellOS SCADA Ingestion")
            .trust_server_certs(true)
            .create_client();

        let session = client.connect_to_endpoint(
            (&self.endpoint_url, SecurityPolicy::None.to_str(), MessageSecurityMode::None),
            IdentityToken::Anonymous,
        ).await?;

        self.client = Some(client);
        self.session = Some(session);
        Ok(())
    }

    async fn read_sensor(
        &self,
        sensor_id: &str,
    ) -> Result<SensorReading, Box<dyn Error>> {
        let session = self.session.as_ref()
            .ok_or("Not connected to OPC UA server")?;

        let config = self.node_map.get(sensor_id)
            .ok_or_else(|| format!("Unknown sensor ID: {}", sensor_id))?;

        // Read OPC UA node value
        let node_id = NodeId::from_str(&config.node_id)?;
        let data_value = session.read_value(&node_id).await?;

        // Translate to domain model
        self.translate_opcua_value_to_reading(sensor_id, &data_value)
    }

    async fn read_sensors(
        &self,
        sensor_ids: &[String],
    ) -> Result<Vec<SensorReading>, Box<dyn Error>> {
        let session = self.session.as_ref()
            .ok_or("Not connected to OPC UA server")?;

        // Batch read OPC UA nodes
        let node_ids: Vec<NodeId> = sensor_ids.iter()
            .filter_map(|id| self.node_map.get(id))
            .filter_map(|config| NodeId::from_str(&config.node_id).ok())
            .collect();

        let data_values = session.read_values(&node_ids).await?;

        let mut readings = Vec::new();
        for (sensor_id, data_value) in sensor_ids.iter().zip(data_values.iter()) {
            let reading = self.translate_opcua_value_to_reading(sensor_id, data_value)?;
            readings.push(reading);
        }

        Ok(readings)
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(session) = &self.session {
            session.disconnect().await;
        }
        self.session = None;
        self.client = None;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(session) = &self.session {
            Ok(session.is_connected())
        } else {
            Ok(false)
        }
    }
}
```

### EIA API Anti-Corruption Layer

```rust
// apps/scada-ingestion/src/adapters/eia_api.rs
use serde::{Deserialize, Serialize};
use reqwest;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// External EIA API response structure
#[derive(Debug, Deserialize)]
struct EiaApiResponse {
    series: Vec<EiaSeriesData>,
}

#[derive(Debug, Deserialize)]
struct EiaSeriesData {
    series_id: String,
    name: String,
    units: String,
    data: Vec<(String, String)>,  // [(date, value)]
}

/// WellOS domain model for commodity prices
#[derive(Debug, Clone)]
pub struct CommodityPrice {
    pub commodity: CommodityType,
    pub price_per_unit: Decimal,
    pub unit: PriceUnit,
    pub date: NaiveDate,
}

#[derive(Debug, Clone)]
pub enum CommodityType {
    OilWTI,       // West Texas Intermediate Crude
    OilBrent,     // Brent Crude
    NaturalGas,   // Henry Hub Natural Gas
}

#[derive(Debug, Clone)]
pub enum PriceUnit {
    PerBarrel,    // Oil
    PerMMBTU,     // Natural gas (million BTU)
}

pub struct EiaApiAdapter {
    api_key: String,
    client: reqwest::Client,
}

impl EiaApiAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch commodity price and translate to domain model
    pub async fn get_commodity_price(
        &self,
        commodity: CommodityType,
    ) -> Result<CommodityPrice, Box<dyn std::error::Error>> {
        // Map domain commodity type to EIA series ID (external knowledge)
        let series_id = match commodity {
            CommodityType::OilWTI => "PET.RWTC.D",      // WTI daily spot price
            CommodityType::OilBrent => "PET.RBRTE.D",   // Brent daily spot price
            CommodityType::NaturalGas => "NG.RNGWHHD.D", // Henry Hub daily spot price
        };

        // Fetch from EIA API
        let url = format!(
            "https://api.eia.gov/v2/seriesid/{}?api_key={}",
            series_id, self.api_key
        );

        let response: EiaApiResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        // Translate external response to domain model
        let series = response.series.first()
            .ok_or("No series data returned")?;

        let (date_str, price_str) = series.data.first()
            .ok_or("No price data available")?;

        // Parse external date format (YYYY-MM-DD)
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

        // Parse price (EIA returns string, convert to Decimal)
        let price = Decimal::from_str_exact(price_str)?;

        // Determine price unit based on commodity
        let unit = match commodity {
            CommodityType::OilWTI | CommodityType::OilBrent => PriceUnit::PerBarrel,
            CommodityType::NaturalGas => PriceUnit::PerMMBTU,
        };

        Ok(CommodityPrice {
            commodity,
            price_per_unit: price,
            unit,
            date,
        })
    }
}
```

---

## Error Handling

### Domain-Specific Errors

```rust
// apps/scada-ingestion/src/adapters/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Sensor not found: {sensor_id}")]
    SensorNotFound { sensor_id: String },

    #[error("Invalid sensor reading: {reason}")]
    InvalidReading { reason: String },

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Authentication failed")]
    AuthenticationFailed,
}

impl From<tokio_modbus::Error> for AdapterError {
    fn from(error: tokio_modbus::Error) -> Self {
        AdapterError::ProtocolError(format!("Modbus error: {}", error))
    }
}

impl From<opcua::client::Error> for AdapterError {
    fn from(error: opcua::client::Error) -> Self {
        AdapterError::ProtocolError(format!("OPC UA error: {}", error))
    }
}
```

### Retry Logic with Exponential Backoff

```rust
// apps/scada-ingestion/src/adapters/retry.rs
use tokio::time::{sleep, Duration};
use std::error::Error;

pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: Error,
{
    let mut retries = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                retries += 1;
                if retries > max_retries {
                    return Err(error);
                }

                // Exponential backoff: 1s, 2s, 4s, 8s
                let delay = Duration::from_secs(2_u64.pow(retries - 1));
                eprintln!("Retry {} after {:?}", retries, delay);
                sleep(delay).await;
            }
        }
    }
}
```

---

## Testing

### Unit Test: Modbus Adapter

```rust
// apps/scada-ingestion/src/adapters/modbus_tcp.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_register_to_reading() {
        let adapter = ModbusTcpAdapter::new("127.0.0.1".to_string(), 502);

        // Simulate raw Modbus register value: 1250 (125.0 PSI with scale factor 0.1)
        let reading = adapter.translate_register_to_reading(
            "well-123-pressure-casing",
            1250,
        ).unwrap();

        assert_eq!(reading.value, 125.0);
        assert!(matches!(reading.unit, MeasurementUnit::PSI));
        assert!(matches!(reading.sensor_type, SensorType::CasingPressure));
        assert!(matches!(reading.quality, DataQuality::Good));
    }

    #[test]
    fn test_data_quality_bad_sensor() {
        let adapter = ModbusTcpAdapter::new("127.0.0.1".to_string(), 502);

        // 0xFFFF indicates sensor disconnected
        let quality = adapter.assess_data_quality(0xFFFF, &SensorType::CasingPressure);
        assert!(matches!(quality, DataQuality::Bad));
    }

    #[test]
    fn test_extract_well_id() {
        let adapter = ModbusTcpAdapter::new("127.0.0.1".to_string(), 502);
        let well_id = adapter.extract_well_id("well-123-pressure-casing");
        assert_eq!(well_id, "well-123");
    }
}
```

---

## Benefits

### 1. Domain Model Purity

✅ Core domain logic is **protocol-agnostic**
✅ No Modbus/OPC UA types leak into business logic
✅ Easy to understand and maintain

### 2. Protocol Flexibility

✅ Swap Modbus for OPC UA without changing domain code
✅ Support multiple protocols simultaneously
✅ Add new protocols by implementing `ProtocolAdapter` trait

### 3. Testability

✅ Mock `ProtocolAdapter` for unit tests
✅ Test domain logic without real SCADA hardware
✅ Integration tests with protocol-specific simulators

### 4. Vendor Independence

✅ Not locked into specific SCADA vendor
✅ Switch vendors without rewriting core application
✅ Support client-specific SCADA systems

### 5. Centralized Translation Logic

✅ All register mappings in one place
✅ Unit conversions standardized
✅ Easy to audit and update

---

## Trade-offs

### Cons

❌ **Additional Layer Complexity** - More code to maintain
❌ **Performance Overhead** - Translation adds latency (minimal in practice)
❌ **Configuration Management** - Register maps must be kept up-to-date
❌ **Learning Curve** - Developers must understand ACL pattern

### Mitigation Strategies

- **Configuration as Code** - Store register maps in version control
- **Code Generation** - Generate adapter code from SCADA system exports
- **Monitoring** - Track translation errors and performance metrics
- **Documentation** - Maintain vendor-specific protocol documentation

---

## Related Patterns

- **Pattern #13: Circuit Breaker Pattern** - Protect against SCADA system failures
- **Pattern #15: Retry Pattern** - Handle transient SCADA communication errors
- **Pattern #28: Frontend Adapter Pattern** - Similar pattern for frontend integrations
- **Pattern #50: SAGA Pattern** - Coordinate multi-step SCADA operations
- **Pattern #83: SCADA Protocol Adapter Pattern** - Detailed SCADA-specific patterns
- **Pattern #88: Industrial IoT Data Pipeline Pattern** - End-to-end SCADA ingestion

---

## Summary

The **Rust Anti-Corruption Layer Pattern** isolates WellOS's clean domain model from messy external SCADA protocols:

✅ **Protocol-agnostic domain** - Core logic independent of Modbus/OPC UA/DNP3
✅ **Trait-based abstraction** - `ProtocolAdapter` trait for all SCADA systems
✅ **Translation layer** - Register addresses → Typed sensor readings
✅ **Error mapping** - Protocol errors → Domain errors
✅ **Testable** - Mock adapters for unit tests
✅ **Flexible** - Add new protocols without changing domain

**Key Takeaway**: External systems should NEVER pollute your domain model. Always translate at the boundary.

---

**Tags**: #rust #anti-corruption-layer #scada #modbus #opcua #dnp3 #protocol-adapter #domain-driven-design #integration
