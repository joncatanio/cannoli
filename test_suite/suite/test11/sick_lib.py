GLOBAL = "I'm a global variable in sick_lib"
PRIV_GLOBAL = "I'm a private global, shhh"

def func(x, y):
   print("func call -- mult: " + str(x * y))
   print("func call calling a private global: " + str(PRIV_GLOBAL))
