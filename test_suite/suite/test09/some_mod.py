import unrelated_mod

def func():
   unrelated_mod.super_duper_function()
   print("calling 'func' in some_mod!")

class some_class():
   def __init__(self):
      self.x = 5
      self.hello = "hello from some_class"

   def print_self(self):
      print("x: " + str(self.x) + " hello: " + str(self.hello))
