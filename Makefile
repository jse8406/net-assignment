all: SimpleEchoTCPServer SimpleEchoTCPClient SimpleEchoUDPClient SimpleEchoUDPServer

SimpleEchoTCPServer: SimpleEchoTCPServer.rs
	rustc SimpleEchoTCPServer.rs -o SimpleEchoTCPServer.exe

SimpleEchoTCPClient: SimpleEchoTCPClient.rs
	rustc SimpleEchoTCPClient.rs -o SimpleEchoTCPClient.exe

SimpleEchoUDPClient: SimpleEchoUDPClient.rs
	rustc SimpleEchoUDPClient.rs -o SimpleEchoUDPClient.exe

SimpleEchoUDPServer: SimpleEchoUDPServer.rs
	rustc SimpleEchoUDPServer.rs -o SimpleEchoUDPServer.exe

