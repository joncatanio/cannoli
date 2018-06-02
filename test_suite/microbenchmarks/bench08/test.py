class Example:
   def __init__(self, x):
      self.x = x

i = 0
while i < 10000000:
   i += 1
   obj = Example(i)
