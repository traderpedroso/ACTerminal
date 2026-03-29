# IceTrader - Real-time Trading Data Feed

Sistema de trading em tempo real com GPUI que recebe dados de mercado via ZeroMQ.

## Estrutura do Projeto

```
icetrader/
├── src/
│   ├── main.rs              # App principal com System Monitor
│   └── datafeed/            # Módulo de dados de mercado
│       ├── mod.rs           # Exports
│       ├── types.rs         # Tipos de dados (Tick, Dom, Quote)
│       ├── tick.rs          # Processamento de ticks
│       ├── dom.rs           # Processamento de DOM
│       ├── quote.rs         # Processamento de Quotes
│       ├── symbols.rs       # Gerenciamento de símbolos
│       └── subscriber.rs    # Conexão ZeroMQ
```

## DataFeed - Uso e APIs

### Símbolos Disponíveis
```rust
use datafeed::get_available_symbols;

let symbols = get_available_symbols();
// ["6L", "6E", "6A", "6B", "6C", "6J", "6S", "6N"]
```

### Tipos de Dados
```rust
use datafeed::DataType;

DataType::Tick   //Ticks (negociações)
DataType::Dom    //Depth of Market (livro de ofertas)
DataType::Quote  //Quotes (dados de mercado)
```

### Criar Subscriber
```rust
use datafeed::DataFeedSubscriber;

let subscriber = DataFeedSubscriber::new();
```

### Assinar/Cancelar Símbolo
```rust
// Assinar
subscriber.subscribe("6E", DataType::Tick);
subscriber.subscribe("6E", DataType::Dom);
subscriber.subscribe("6E", DataType::Quote);

// Cancelar assinatura
subscriber.unsubscribe("6E", DataType::Tick);

// Verificar se está assinado
let is_subbed = subscriber.is_subscribed("6E", DataType::Tick);
```

### Iniciar Receiver (thread ZMQ fica sempre conectada)
```rust
let mut rx = subscriber.start();

// Receber dados: (symbol, data_type, json_string)
while let Some((symbol, data_type, data)) = rx.blocking_recv().await {
    // processar dados
}
```

### Shutdown
```rust
subscriber.shutdown();
```

## Processamento de Dados

### Tick
```rust
use datafeed::{tick, TickData};

// Parsear mensagem
let msg = tick::parse_message(json)?;

// Processar com lógica de direção (BUY_MARKET, SELL_MARKET, etc)
let ticks = tick::process(json)?; // -> Vec<TickData>

// Para JSON formatado
let json_str = tick::to_json_string(json)?;
```

### DOM
```rust
use datafeed::{dom, DomData};

let dom_data = dom::process(json)?; // -> DomData
let json_str = dom::to_json_string(json)?;
```

### Quote
```rust
use datafeed::{quote, QuoteData};

let quote_data = quote::process(json)?; // -> QuoteData
let json_str = quote::to_json_string(json)?;
```

## Tipos de Dados (types.rs)

### TickData
```rust
pub struct TickData {
    pub id: i64,
    pub time: i64,
    pub price_ticks: i64,
    pub price: f64,
    pub size: u64,
    pub side: String,        // "BUY_MARKET", "SELL_MARKET", "BUY_PENDING", "SELL_PENDING", "MID_PRICE_TRADE"
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
}
```

### DomData
```rust
pub struct DomData {
    pub contract_id: i64,
    pub timestamp: String,
    pub bids: Vec<DomEntry>,    // [{ price, size }, ...]
    pub offers: Vec<DomEntry>,  // [{ price, size }, ...]
}
```

### QuoteData
```rust
pub struct QuoteData {
    pub timestamp: String,
    pub contract_id: i64,
    pub entries: QuoteEntries,
}

pub struct QuoteEntries {
    pub bid: Option<QuoteEntry>,
    pub offer: Option<QuoteEntry>,
    pub trade: Option<QuoteEntry>,
    pub low_price: Option<QuoteEntry>,
    pub high_price: Option<QuoteEntry>,
    pub opening_price: Option<QuoteEntry>,
    pub settlement_price: Option<QuoteEntry>,
    pub open_interest: Option<QuoteEntry>,
    pub total_trade_volume: Option<QuoteEntry>,
}
```

## ZeroMQ

- Conexão: `tcp://127.0.0.1:5555`
- Protocolo: SUB socket
- Topic prefix: `tick.`, `dom.`, `quote.`
- Exemplo: `tick.6E`, `dom.6E`, `quote.6E`

## Commands Úteis

```bash
# Build
cargo build

# Run
cargo run
```
