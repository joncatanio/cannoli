a = [1, 2, 3]
a.append(15)

print(a)
b = a
b.append("hi")
print(a)

a.append(['a', 'b', True, False])

print("a: " + str(a))
print("b: " + str(b))

a = [[1, 2], [3, 4], [5, 6]]
print(a)
