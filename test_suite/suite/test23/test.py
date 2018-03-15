import sys

print(sys.argv)

a = open("sample.txt", "r")
b = open("output.txt", "w")

for n, line in enumerate(a):
   print(n, line)

for n, line in enumerate(a):
   print(n, line)

print("write this out", file=b)

a.close()
b.close()
