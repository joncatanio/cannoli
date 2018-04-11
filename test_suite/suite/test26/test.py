class Test:
   x = 5
   y = 2

   def __init__(self, y, z):
      self.first = y
      self.second = z

      if True:
         self.true = True
      else:
         self.true = False
         self.false = True

      (self.tup1, self.tup2) = 500, 600

   def some_func(self, a, b, c):
      print(a, b, c)

   def class_func(b):
      print("Class func", b)

a = Test("first", "second")

print(a.x, a.y, a.first, a.second, a.true, a.tup1, a.tup2)
a.some_func("a", "b", "c")

Test.class_func("woo")
