# Assignment Overview  
Rust를 이용해 TCP 기반의 서버와 클라이언트를 구현하며 네트워크 프로토콜의 동작 방식과 멀티스레딩, 공유 상태 관리, 시그널 처리 등을 학습하는 과제  


  
<img width="2500" height="1700" alt="image" src="https://github.com/user-attachments/assets/7dc3cb77-d7ee-4e5a-9ed5-f3da9a027cff" />

**Build TCP & UDP Server/Client using Rust**

# Requirements

## Client Requirements

### 1. Cargo 기반 프로젝트 구성
- Cargo를 이용해 Rust 프로젝트를 구성하고 실행해야 합니다.

### 2. Graceful Shutdown
- Ctrl+C 입력 시 클라이언트가 즉시 종료되지 않고  
  OS Signal(SIGINT)을 처리하여 서버와의 연결을 안전하게 종료한 뒤 종료해야 합니다.

---

## Server Requirements

### 1. Unique Client ID  
서버에 접속한 각 클라이언트에게 고유한 ID를 부여해야 합니다.

### 2. Connection Logging  
클라이언트가 접속하거나 종료될 때 **시간 / ID / 접속자 수**를 출력합니다.

#### 연결 시

[Time: HH:MM:SS] Client <ID> connected. Number of clients connected = N


#### 종료 시

[Time: HH:MM:SS] Client <ID> disconnected. Number of clients connected = M  


### 3. Periodic Logging (10 sec)  
10초마다 서버에 접속해 있는 클라이언트 수를 출력합니다.


---

# Implementation Details

## Server Features

### 1. Multi-threading  
- `std::thread`를 이용하여 **각 클라이언트 연결을 개별 스레드**로 처리합니다.

### 2. Shared State Management  
- `Arc<Mutex<...>>` 를 사용하여 클라이언트 목록과 요청 카운트를 공유합니다.

### 3. Protocol Handler (서버 명령 처리)

| Option | Description |
|--------|-------------|
| **OPT1** | 텍스트 대문자 변환 (Upper-case conversion) |
| **OPT2** | 서버 가동 시간 조회 |
| **OPT3** | 클라이언트 IP 및 Port 확인 |
| **OPT4** | 총 처리된 요청 수 조회 |
| **OPT5** | 연결 종료 요청 |

### 4. Signal Handling  
- `ctrlc` 크레이트를 사용하여 서버가 종료되기 전  
  **모든 클라이언트 연결을 안전하게 종료한 뒤 종료**해야 합니다.

---

## Client Features

### 1. Interactive Menu  
- 사용자가 명령을 선택해 서버로 요청을 보내도록 UI 메뉴를 제공합니다.

### 2. RTT Measurement  
- 서버로 요청을 보내고 응답이 오기까지 걸린 시간을 **밀리초(ms)** 단위로 측정하여 출력합니다.

### 3. Server Monitoring  
- 별도의 스레드에서 서버와의 연결 상태를 감시합니다.
- 서버가 종료되면 클라이언트도 자동 종료해야 합니다.

---

# How to Run

## Server
```bash
cd multi_tcp_server
cargo run
```
## Client
```bash
cd multi_tcp_client
cargo run
```
# Screenshots for feature 3 and 4  

<img width="1909" height="1008" alt="image" src="https://github.com/user-attachments/assets/a94f5d99-c3e0-4a14-9d23-b3efef672842" />
<img width="1908" height="1000" alt="image" src="https://github.com/user-attachments/assets/2aca8724-e8ec-48a4-a1d4-78b243be3bdc" />

---

## 과제 P2P

### P2P 채팅 애플리케이션 개요
Rust를 이용해 P2P(Peer-to-Peer) 기반 채팅 시스템을 구현하며, 노드 간 직접 통신과 멀티스레딩을 학습하는 과제입니다.

### 주요 기능

#### 1. 노드 기반 채팅
- 각 노드는 고유한 ID(1-4)를 가지며, 하드코딩된 피어 주소로 연결됩니다.
- 노드 ID와 닉네임(16자 이하)을 인자로 받아 실행합니다.

#### 2. 명령어 지원
- `\help`: 사용 가능한 명령어 목록 출력
- `\list`: 현재 연결된 피어 목록 표시
- `\quit`: 모든 피어에게 연결 종료 메시지를 보내고 프로그램 종료
- 일반 메시지: 입력한 텍스트를 모든 연결된 피어에게 브로드캐스트

#### 3. 멀티스레딩
- 각 피어 연결을 별도의 스레드로 처리하여 동시 채팅 가능
- 비동기 I/O를 위해 Tokio 크레이트 사용

#### 4. 연결 관리
- 피어 주소는 상수로 정의되어 있으며, localhost 또는 원격 호스트로 설정 가능
- 연결 실패 시 재시도 로직 포함
- 각 노드당 최대 incoming 및 outgoing 연결 수는 3개로 제한
- 연결된 피어 목록을 각 노드가 자체 리스트에 저장하여 관리

### 실행 방법
```bash
cd p2p_chat
cargo run <node_id> <nickname>
```
예시:
```bash
cargo run 1 Alice
```

### 호스트 설정
- 기본적으로 localhost(127.0.0.1)로 설정 가능
- 필요 시 원격 호스트(nsl5.cau.ac.kr 등)로 변경 가능

# Screenshots for connecting and feature \list  

  유저 4명 연결
  <img width="1917" height="1014" alt="image" src="https://github.com/user-attachments/assets/5c4f5e02-b36f-42d4-af58-201b7687424f" />  
  \list 기능
  <img width="1909" height="1013" alt="image" src="https://github.com/user-attachments/assets/636b60db-17f2-4266-8481-62e752ed7649" />


