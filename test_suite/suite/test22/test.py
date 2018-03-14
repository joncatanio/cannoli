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
