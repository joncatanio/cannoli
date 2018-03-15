import sys

print(sys.argv)

a = open("sample.txt", "r")

for n, line in enumerate(a):
   print(n, line)

for n, line in enumerate(a):
   print(n, line)
