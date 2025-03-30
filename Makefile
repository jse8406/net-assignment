all: my_tcp_server my_tcp_client my_udp_client my_udp_server

my_tcp_server: my_tcp_server.rs
	rustc my_tcp_server.rs -o my_tcp_server.exe

my_tcp_client: my_tcp_client.rs
	rustc my_tcp_client.rs -o my_tcp_client.exe


my_udp_server: my_udp_server.rs
	rustc my_udp_server.rs -o my_udp_server.exe

my_udp_client: my_udp_client.rs
	rustc my_udp_client.rs -o my_udp_client.exe