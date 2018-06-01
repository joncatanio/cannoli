class Example:
   def __init__(self):
      self.x = 1

def func(obj : Example):
   obj.x

obj = Example()

i = 0
while i < 10000000:
   i += 1
   func(obj)
