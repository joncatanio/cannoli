#include <stdio.h>
#include <stdlib.h>

typedef enum {
   INT,
   FLOAT
} Types;

typedef struct {
   union {
      int   int_val;
      float float_val;
   };
   Types type;
} Type;

Type* cons_int(int val) {
   Type* res_type = malloc(sizeof(Type));
   res_type->type = INT;
   res_type->int_val = val;

   return res_type;
}

Type* cons_float(float val) {
   Type* res_type = malloc(sizeof(Type));
   res_type->type = FLOAT;
   res_type->float_val = val;

   return res_type;
}

Type* add(Type* a, Type* b) {
   Type* res_type = malloc(sizeof(Type));

   if (a->type == INT) {
      if (b->type == INT) {
         res_type->type = INT;
         res_type->int_val = a->int_val + b->int_val;

         printf("INT: %d\n", res_type->int_val);
      } else if (b->type == FLOAT) {
         res_type->type = FLOAT;
         res_type->float_val = a->int_val + b->float_val;

         printf("FLOAT: %f\n", res_type->float_val);
      }
   }

   return res_type;
}
