all: SimpleEchoTCPServer SimpleEchoTCPClient SimpleEchoUDPClient SimpleEchoUDPServer

SimpleEchoTCPServer: SimpleEchoTCPServer.rs
	rustc SimpleEchoTCPServer.rs -o SimpleEchoTCPServer

SimpleEchoTCPClient: SimpleEchoTCPClient.rs
	rustc SimpleEchoTCPClient.rs -o SimpleEchoTCPClient

SimpleEchoUDPClient: SimpleEchoUDPClient.rs
	rustc SimpleEchoUDPClient.rs -o SimpleEchoUDPClient

SimpleEchoUDPServer: SimpleEchoUDPServer.rs
	rustc SimpleEchoUDPServer.rs -o SimpleEchoUDPServer

