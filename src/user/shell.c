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
    else
      print_dbg("unknown command\n");
  }
}
