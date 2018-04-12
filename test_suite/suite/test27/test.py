class Test:
   def __init__(self, x):
      self.x = x
      self.y = "hello"

   def some_func(self, other):
      print(self.x, self.y)
      print(other.x, other.y)

def func(one: Test, two: Test):
   one.x = two.x
   two.y = one.y

   print(one.x, one.y)
   print(two.x, two.y)

a : Test = Test(10)
b : Test = Test(5)
c : Test = b

a.some_func(c)
func(a, c)

print(a.x, a.y)
print(b.x, b.y)
print(c.x, c.y)

b.x = 499
print(c.x, c.y)
