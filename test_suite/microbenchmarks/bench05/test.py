lst = []
i = 0
while i < 10000000:
   lst.append(i)
   i += 1

while lst:
   lst.pop()
