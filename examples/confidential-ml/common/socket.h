void listen_and_receive_data(const char* host_name, int port, void (*callback)(char *, int));
void send_data(const char* host_name, int port, unsigned char *msg, int len);