#include <stdio.h>
#include <stdlib.h>

typedef enum {
   INT
} Types;

typedef struct {
   union {
      int int_val;
   };
   Types type;
} Type;

Type* cons_int(int val) {
   Type* res_type = malloc(sizeof(Type));
   res_type->type = INT;
   res_type->int_val = val;

   return res_type;
}

Type* add(Type* a, Type* b) {
   Type* res_type = malloc(sizeof(Type));

   if (a->type == INT) {
      if (b->type == INT) {
         res_type->type = INT;
         res_type->int_val = a->int_val + b->int_val;

         printf("INT: %d\n", res_type->int_val);
      }
   }

   return res_type;
}
