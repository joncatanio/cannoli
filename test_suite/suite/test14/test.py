def get_int(x):
   return x

def slice_it_up(a):
   print(a[1:5])
   print(a[1:12])
   print(a[1:24])
   print(a[-5:10])
   print(a[-1:])
   print(a[-20000:])
   print(a[1:-2])
   print(a[1:0])
   print(a[1:])
   print(a[:5])
   print(a[c:])
   print(a[:get_int(9)])
   print(a[-111111:-222222222])
   print(a[12:0:-1])
   print(a[5:1:-1])
   print(a[5:1:-2])
   print(a[5:1:-20])
   print(a[::-1])
   print(a[1:4:-1])
   print(a[-1:-4:1])
   print(a[-1:-4:2])
   print(a[-1::2])
   print(a[-1:-4:-1])
   print(a[-1::-1])
   print(a[1:-5:-2])
   print(a[:-5:-2])
   print(a[10:-5:-2])
   print(a[::-2])
   print(a[0:-2:-1])

a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
b = a[:]

c = 4

b.append(-1)
print(a)
print(b)

slice_it_up(a)
slice_it_up([1])
slice_it_up([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
slice_it_up([])

j = [1]
print(j[-1::-1])
print(j[-1:-4:-1])
print(j[-1::-1])
print(j[1:-5:-2])
print(j[:-5:-2])
print(j[10:-5:-2])
print(j[::-2])
print(j[0:-2:-1])

# Tuple slices
slice_it_up((1,))
slice_it_up((1, 2, 3, 4, 'jon', True))
slice_it_up((1, 'somethang', 4, 'jon', True, False))
slice_it_up(())
