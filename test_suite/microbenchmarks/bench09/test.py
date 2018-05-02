lst = []
i = 0
while i < 100:
   i += 1
   lst.append(i)

i = 0
while i < 1000000:
   i += 1
   new_lst = [x for x in lst]
