a = [1, 2, 3, 4, 5, 6]
b = ["jon", "catanio", "test", "it", "all"]
[print(str(x) + " " + y) for x in a for y in b]
[print(x) for x in [y for y in b]]

def operate(x):
   a = x + 6 - 4 * 2
   print(a)
   return a

g = [operate(x) for x in a]

print("LIST: " + str(g))
print([x + 5 for x in a])

a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
b = [90, 100, 111, 3213, 32132, 4244, 4255, 212, 21455, 215121]
c = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4]
result = [x + y for x in a if x % 2 if x > 3 for y in b if y % 2 == 0 for z in c]

print(result)

def cast(val, x):
   return val * x

val = 15

print([(x, cast(val, x)) for x in a])
