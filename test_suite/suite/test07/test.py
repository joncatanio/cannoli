class Node:
   def __init__(self, val, next):
      self.val = val
      self.next = next

lst = Node(1, Node(5, Node(8, Node(-1, None))))
temp = lst

while temp:
   print(temp.val)
   temp = temp.next
