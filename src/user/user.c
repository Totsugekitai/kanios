#include "user.h"

extern char __stack_top[];

long long syscall(uint64_t sysno, uint64_t arg0, uint64_t arg1, uint64_t arg2) {
  register uint64_t a0 __asm__("a0") = arg0;
  register uint64_t a1 __asm__("a1") = arg1;
  register uint64_t a2 __asm__("a2") = arg2;
  register uint64_t a3 __asm__("a3") = sysno;

  __asm__ __volatile__("ecall"
                       : "=r"(a0)
                       : "r"(a0), "r"(a1), "r"(a2), "r"(a3)
                       : "memory");

  return a0;
}

void putchar(char ch) { syscall(SYS_PUTCHAR, ch, 0, 0); }

int getchar(void) { return syscall(SYS_GETCHAR, 0, 0, 0); }

int readfile(const char *filename, char *buf, uint64_t len) {
  return syscall(SYS_READFILE, (uint64_t)filename, (uint64_t)buf, len);
}

int writefile(const char *filename, const char *buf, uint64_t len) {
  return syscall(SYS_WRITEFILE, (uint64_t)filename, (uint64_t)buf, len);
}

__attribute__((noreturn)) void exit(void) {
  syscall(SYS_EXIT, 0, 0, 0);
  for (;;)
    ;
}

__attribute__((section(".text.start"))) __attribute__((naked)) void start(
    void) {
  __asm__ __volatile__(
      "mv sp, %[stack_top]\n"
      "call main\n"
      "call exit\n" ::[stack_top] "r"(__stack_top));
}
