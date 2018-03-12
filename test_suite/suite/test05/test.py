class First:
   def __init__(self, second):
      self.x = 5
      self.second = second

class Second:
   def __init__(self):
      self.y = 10

   def print_content(self):
      print("print_content -- self.y: " + str(self.y))

def main():
   b = Second()
   a = First(b)

   a.second.print_content()

if __name__ == '__main__':
   main()
