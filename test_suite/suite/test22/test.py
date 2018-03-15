a = 1
(a, b) = 1, 2
(c, d) = "hello", "shalom"

print(a, b, c, d)

(var, bar, tar) = [True, False, "Hellooooo"]

print(var, bar, tar)

for a, ((b, c), d) in [(1, ((2, 3), 4))]:
   print(a, b, c, d)

(a, (b, (c, d), e), f, g) = ("jon", (5, (True, False), "cat"), 0, -1)
print(a, b, c, d, e, f, g)

class SomeClass:
   a = 4
   def jon(self):
      b = 10
      print(SomeClass.a, b)

SomeClass().jon()

a = 2
a += 4
print(a)
a -= 4
print(a)
a *= 2
print(a)
a = 2.1
a /= 2
print(a)
a = 12
a %= 10
print(a)
a **= 2
print(a)
a = 3
a <<= 1
print(a)
a = 3
a >>= 1
print(a)
a = 3
a |= 3
print(a)
a = 6
a ^= 5
print(a)
a = 9
a &= 7
print(a)
