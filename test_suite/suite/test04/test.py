class Test:
   var = 2
   def __init__(self, x, y):
      self.x = x
      self.y = y

   def method(self):
      print("method: self.x = " + str(self.x))
      print("method: self.y = " + str(self.y))
      print("Updating self.x")
      self.x = "`method` updated my value"

   def swapxy(self):
      temp = self.x
      self.x = self.y
      self.y = temp

int_val = 5

obj1 = Test("x value", "y value")
obj2 = obj1

print("Updating obj1.var ...")
obj1.var = 4

print("obj1.var: " + str(obj1.var))
print("obj2.var: " + str(obj2.var))

print("Updating obj2.x ...")
print("PRE obj1.x: " + str(obj1.x))
print("PRE obj2.x: " + str(obj2.x))
obj2.x = "changed string"
print("POST obj1.x: " + str(obj1.x))
print("POST obj2.x: " + str(obj2.x))

print("Assign obj2.var to variable 'a'")
a = obj2.var
print("a: " + str(a))

print("Modify 'a' to show that obj2.var won't be effected")
a = 15
print("a: " + str(a))
print("obj2.var: " + str(obj2.var))

print("Calling obj1.method() ...")
obj1.method()
print("State of obj1 & obj2 after call")
print("obj1.x: " + str(obj1.x) + " obj1.y: " + str(obj1.y) + " obj1.var: " + str(obj1.var))
print("obj2.x: " + str(obj2.x) + " obj2.y: " + str(obj2.y) + " obj2.var: " + str(obj2.var))

print("Calling obj1.swapxy() ...")
obj1.swapxy()
print("obj1.x: " + str(obj1.x) + " obj1.y: " + str(obj1.y) + " obj1.var: " + str(obj1.var))
print("obj2.x: " + str(obj2.x) + " obj2.y: " + str(obj2.y) + " obj2.var: " + str(obj2.var))
