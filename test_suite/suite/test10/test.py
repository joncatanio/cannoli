def add(x, y):
   return x + y

print(add(5, 2))

def func(a):
   if a:
      print("value is true!")
      return Some()
   else:
      return None

class Some:
   def __init__(self):
      self.x = "hello"

   def say_hello(self, name):
      print(self.x + " " + str(name))

func(True).say_hello("Jon")
print(func(False))
