import math

a = [1, 2, 3, 4, "True", True, False, -5, (1, 2)]

print(1 in [1, 2] in [[1, 2], 3, 4])
print("String in list lookup")
print("True" in a)
print("True" not in a)
print(-5 in a)

if (1, 2) in a:
   print("Woa nice, it is!")

if "jon" not in a:
   print("jon is not in a")

print([1, 2, 3] in [[1, 2, 3], [4, 5, 6]])
print("jon" in ["jon", "cat", 1, 2, "jon", True, 0, -1])
print("jon" not in ["jon", "cat", 1, 2, "jon", True, 0, -1])

print(1 > 2)
print(1 < 2)
print([1, "nice", True] == [1, "nice", True])
print([1, "nice", True] == [3, "nice", True])
print("!= test")
print([1, "nice", True] != [3, "nice", True])

print(5 >= 3)
print(5 <= 3)
print(-2 <= 3)
print(-2 >= 3)

print("" in "some string")
print("substring" in "this string has a substring woa")

print(1 ** 2)
print(0 ** 5)

# uncomment and visually check, the floating point values are slightly
# different between Rust and Python
#print(math.sqrt(3))
#print(6 ** -2)
#print(6 ** -2.4)
#print(4.2 ** 8)
#print(4.5 ** 2.4)
#print(9.2 ** -1.2)
