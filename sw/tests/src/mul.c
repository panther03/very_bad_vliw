int getchar();
int multiply(int x, int y)
{
  int ret = 0;
  for (int i = 0; i < 32; i++)
  {
    if (((y >> i) & 1) == 1)
    {
      ret += (x << i);
    }
  }
  return ret;
}
int putchar(int c);
int exit(int c);

int main()
{
  int a = 32;
  int b = 69;
  int result = multiply(a, b);
  if (result == 2208)
  {
    putchar('O');
    putchar('k');
    /* char *s = "Hello, world!\n"; */
    /* char *p; */
    /* for (p = s; p < s + 14; p++) */
    /*   putchar(*p); */
    exit(0);
  }
  else
  {
    char *s = "Jello, world!\n";
    char *p;
    for (p = s; p < s + 14; p++)
      putchar(*p);
    putchar(0x30 + result);
    exit(1);
  }
  return 0;
}
