# Rust-based Protocols Client-Server Ping-Pong Bandwidth Tests

This repository contains a set of Rust-based protocols client-server ping-pong bandwidth tests. These tests are designed to measure the bandwidth performance of various protocols implemented in Rust. The goal of these tests is to select the most suitable protocol for the atm0s-sdn project.

## Protocols

- TCP
- UDP
- QUIC: QUINN
- QUIC: TQUIC

## Runtime

- Native
- Mio
- Tokio
- Monoio
- Async-std

## Testing environmenet

Running both client and server in same instance: Github CodeSpace 2-CPU

## Result


### Tokio - Quinn

client: 75-80%
server: 75-80%

```bash
166 MB/s
171 MB/s
175 MB/s
195 MB/s
```

### Mio-TQuic

client: 75-80%
server: 75-80%

```bash
58 MB/s
66 MB/s
63 MB/s
67 MB/s
```

### Native TCP

client: 75-80%
server: 75-80%

```bash
1785 MB/s
1867 MB/s
1859 MB/s
1986 MB/s
```

### Native UDP

simulate 100 channels

client: 80-90%
server: 80-90%

```bash
178 MB/s
189 MB/s
217 MB/s
214 MB/s
```

### Monoio UDP

simulate 100 channels

client: 80-85%
server: 80-85%

```bash
136 MB/s
141 MB/s
135 MB/s
139 MB/s
```

### Tokio UDP

simulate 100 channels

client: 80-85%
server: 80-85%

```bash
65 MB/s
82 MB/s
65 MB/s
49 MB/s
```

### Mio UDP

simulate 100 channels

client: 80-85%
server: 80-85%

```bash
164 MB/s
180 MB/s
164 MB/s
180 MB/s
```

### Async-std UDP

simulate 100 channels

client: 80-90%
server: 80-90%

```bash
147 MB/s
187 MB/s
192 MB/s
192 MB/s
```