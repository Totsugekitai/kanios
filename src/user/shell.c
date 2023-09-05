#include "user.h"

void main(void) {
  while (1) {
  prompt:
    print_dbg("> ");
    char cmdline[128] = {0};
    for (int i = 0;; i++) {
      char ch = getchar();
      putchar(ch);
      if (i == sizeof(cmdline) - 1) {
        print_dbg("command line too long\n");
        goto prompt;
      } else if (ch == '\r') {
        print_dbg("\n");
        cmdline[i] = '\0';
        break;
      } else {
        cmdline[i] = ch;
      }
    }

    if (strcmp(cmdline, "hello") == 0)
      print_dbg("Hello world from shell!\n");
    else if (strcmp(cmdline, "exit") == 0)
      exit();
    else if (strcmp(cmdline, "readfile") == 0) {
      char buf[128] = {0};
      int len = readfile("hello.txt", buf, sizeof(buf));
      buf[len] = '\0';
      print_dbg(buf);
    } else if (strcmp(cmdline, "writefile") == 0)
      writefile("hello.txt", "Hello from shell!\n", 19);
    else
      print_dbg("unknown command\n");
  }
}
