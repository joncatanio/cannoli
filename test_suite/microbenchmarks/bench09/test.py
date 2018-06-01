class Example:
   def __init__(self):
      self.num = 0
      self.x = "This obj"

   def method(self, num):
      self.num = num

obj = Example()
i = 0
while i < 10000000:
   i += 1
   obj.method(i)
