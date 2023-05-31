#include <stdio.h>
#include <stdlib.h>
#include <netdb.h>
#include <netinet/in.h>
#include <sys/types.h>
#include <unistd.h>
#include <string.h>
#include <sys/socket.h>
#include <stdbool.h>
#include <time.h>

// server-side functions
static struct sockaddr_in* init_sockaddr_in(uint16_t port_number) {
  struct sockaddr_in *socket_address = (struct sockaddr_in *)malloc(sizeof(struct sockaddr_in));
  memset(socket_address, 0, sizeof(*socket_address));
  socket_address -> sin_family = AF_INET;
  socket_address -> sin_addr.s_addr = htonl(INADDR_ANY);
  socket_address -> sin_port = htons(port_number);
  return socket_address;
}

static char* process_operation(char *input) {
  size_t n = strlen(input) * sizeof(char);
  char *output = (char *)malloc(n);
  memcpy(output, input, n);
  return output;
}

static bool open_server_socket(const char* host_name, int port, int* soc) {
  struct addrinfo hints;
  struct addrinfo *result, *rp;
  int sfd, s;

  memset(&hints, 0, sizeof(struct addrinfo));
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;
  hints.ai_flags = AI_PASSIVE;
  hints.ai_protocol = 0;
  hints.ai_canonname = NULL;
  hints.ai_addr = NULL;
  hints.ai_next = NULL;

  char port_str[16] = {};
  sprintf(port_str, "%d", port);

  s = getaddrinfo(host_name, port_str, &hints, &result);
  if (s != 0) {
    fprintf(stderr, "getaddrinfo: %s\n", gai_strerror(s));
    return false;
  }

  for (rp = result; rp != NULL; rp = rp->ai_next) {
    int option = 1;
    sfd = socket(rp->ai_family, rp->ai_socktype,
                 rp->ai_protocol);
    if (sfd == -1)
      continue;
    setsockopt(sfd, SOL_SOCKET, SO_REUSEADDR, &option, sizeof(option));

    if (bind(sfd, rp->ai_addr, rp->ai_addrlen) == 0)
      break;

    close(sfd);
  }

  if (rp == NULL) {
    fprintf(stderr, "Could not bind\n");
    return false;
  }

  freeaddrinfo(result);

  if (listen(sfd, 10) != 0) {
    printf("open_server_socket: cant listen\n");
    return false;
  }

  *soc = sfd;
  return true;
}

// public functions
void listen_and_receive_data(const char* host_name, int port, void (*callback)(char *, int)) {
  int server_fd = -1;
  if (!open_server_socket(host_name, port, &server_fd)) {
    printf("server_dispatch: Can't open server socket\n");
    return;
  }

  while (1) {
    struct sockaddr_in client_sockaddr;
    unsigned int client_socklen = sizeof(struct sockaddr_in);

    char buffer[2048] = {0,};
    int client_fd = accept(server_fd, (struct sockaddr *) &client_sockaddr, &client_socklen);

    int n = read(client_fd, buffer, sizeof(buffer));
    callback(buffer, n);

    close(client_fd);
  }
}

int send_data(const char* host_name, int port, unsigned char *msg, int len) {
  int sockfd, portno, n;
  struct sockaddr_in serv_addr;
  char buffer[256] = {0,};

  struct addrinfo hints;
  struct addrinfo *result, *rp;
  int sfd, s;

  memset(&hints, 0, sizeof(struct addrinfo));
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;
  hints.ai_flags = 0;
  hints.ai_protocol = 0;

  char port_str[16] = {};
  sprintf(port_str, "%d", port);

  s = getaddrinfo(host_name, port_str, &hints, &result);
  if (s != 0) {
    fprintf(stderr, "getaddrinfo: %s\n", gai_strerror(s));
    return -1;
  }

  for (rp = result; rp != NULL; rp = rp->ai_next) {
    sfd = socket(rp->ai_family, rp->ai_socktype, rp->ai_protocol);
    if (sfd == -1)
      continue;

    if (connect(sfd, rp->ai_addr, rp->ai_addrlen) != -1)
      break;

    close(sfd);
  }

  n = write(sfd, msg, len);
  if (n <= 0) {
    printf("write error\n");
    return -1;
  }

  close(sfd);
  return n;
}