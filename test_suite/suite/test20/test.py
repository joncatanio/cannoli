print("Built-In Test: len()")
print(len("this is a string"))
print(len([1, 2, "list", "this", "self", True, False, ["inner", "list"]]))
print(len(("tuple", "test", 1, 2, 3)))
print(len(""))
print(len(()))
print(len([]))

print("Built-In Test: min()")
print(min([5, 2, 7, 1, -20, 15, 90]))
print(min([5, 2, 7, 1, -20, 15, 90, -54.2]))
print(min([-2005, 2, 7, 1, -20, 15, 90, -54.2]))
print(min((6, 7, -2)))
print(min(1, 2))
print(min(-21, -2))
print(min(52.0, 1))
print(min(52, 1.0, -12))
print(min(52, 1.0, 12, 3, 5, 7, 21.2, 4, 0, 2))

print("Built-In Test: int()")
print(int("       102       \n"))
print(int("501"))
print(int("-501"))
print(int("0"))

print("Built-In Test: float()")
print(float("1.23"))
print(float("     -12345.1\n"))
print(float("1e-003"))
print(float("2.5E-1"))
print(float(".5     "))

print("Built-In Test: enumerate()")
a = ["jon", "some", 1, 2, 3, True, False]
for x, y in enumerate(a):
   print(x, y)
for x, y in enumerate(a, -5):
   print(x, y)

print([(x, y) for x, y in enumerate(a)])
