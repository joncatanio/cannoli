; ModuleID = 'lib.c'
source_filename = "lib.c"
target datalayout = "e-m:o-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.13.0"

%struct.Type = type { %union.anon, i32 }
%union.anon = type { i32 }

@.str = private unnamed_addr constant [9 x i8] c"INT: %d\0A\00", align 1

; Function Attrs: noinline nounwind ssp uwtable
define %struct.Type* @cons_int(i32) #0 {
  %2 = alloca i32, align 4
  %3 = alloca %struct.Type*, align 8
  store i32 %0, i32* %2, align 4
  %4 = call i8* @malloc(i64 8) #3
  %5 = bitcast i8* %4 to %struct.Type*
  store %struct.Type* %5, %struct.Type** %3, align 8
  %6 = load %struct.Type*, %struct.Type** %3, align 8
  %7 = getelementptr inbounds %struct.Type, %struct.Type* %6, i32 0, i32 1
  store i32 0, i32* %7, align 4
  %8 = load i32, i32* %2, align 4
  %9 = load %struct.Type*, %struct.Type** %3, align 8
  %10 = getelementptr inbounds %struct.Type, %struct.Type* %9, i32 0, i32 0
  %11 = bitcast %union.anon* %10 to i32*
  store i32 %8, i32* %11, align 4
  %12 = load %struct.Type*, %struct.Type** %3, align 8
  ret %struct.Type* %12
}

; Function Attrs: allocsize(0)
declare i8* @malloc(i64) #1

; Function Attrs: noinline nounwind ssp uwtable
define %struct.Type* @add(%struct.Type*, %struct.Type*) #0 {
  %3 = alloca %struct.Type*, align 8
  %4 = alloca %struct.Type*, align 8
  %5 = alloca %struct.Type*, align 8
  store %struct.Type* %0, %struct.Type** %3, align 8
  store %struct.Type* %1, %struct.Type** %4, align 8
  %6 = call i8* @malloc(i64 8) #3
  %7 = bitcast i8* %6 to %struct.Type*
  store %struct.Type* %7, %struct.Type** %5, align 8
  %8 = load %struct.Type*, %struct.Type** %3, align 8
  %9 = getelementptr inbounds %struct.Type, %struct.Type* %8, i32 0, i32 1
  %10 = load i32, i32* %9, align 4
  %11 = icmp eq i32 %10, 0
  br i1 %11, label %12, label %38

; <label>:12:                                     ; preds = %2
  %13 = load %struct.Type*, %struct.Type** %4, align 8
  %14 = getelementptr inbounds %struct.Type, %struct.Type* %13, i32 0, i32 1
  %15 = load i32, i32* %14, align 4
  %16 = icmp eq i32 %15, 0
  br i1 %16, label %17, label %37

; <label>:17:                                     ; preds = %12
  %18 = load %struct.Type*, %struct.Type** %5, align 8
  %19 = getelementptr inbounds %struct.Type, %struct.Type* %18, i32 0, i32 1
  store i32 0, i32* %19, align 4
  %20 = load %struct.Type*, %struct.Type** %3, align 8
  %21 = getelementptr inbounds %struct.Type, %struct.Type* %20, i32 0, i32 0
  %22 = bitcast %union.anon* %21 to i32*
  %23 = load i32, i32* %22, align 4
  %24 = load %struct.Type*, %struct.Type** %4, align 8
  %25 = getelementptr inbounds %struct.Type, %struct.Type* %24, i32 0, i32 0
  %26 = bitcast %union.anon* %25 to i32*
  %27 = load i32, i32* %26, align 4
  %28 = add nsw i32 %23, %27
  %29 = load %struct.Type*, %struct.Type** %5, align 8
  %30 = getelementptr inbounds %struct.Type, %struct.Type* %29, i32 0, i32 0
  %31 = bitcast %union.anon* %30 to i32*
  store i32 %28, i32* %31, align 4
  %32 = load %struct.Type*, %struct.Type** %5, align 8
  %33 = getelementptr inbounds %struct.Type, %struct.Type* %32, i32 0, i32 0
  %34 = bitcast %union.anon* %33 to i32*
  %35 = load i32, i32* %34, align 4
  %36 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str, i32 0, i32 0), i32 %35)
  br label %37

; <label>:37:                                     ; preds = %17, %12
  br label %38

; <label>:38:                                     ; preds = %37, %2
  %39 = load %struct.Type*, %struct.Type** %5, align 8
  ret %struct.Type* %39
}

declare i32 @printf(i8*, ...) #2

attributes #0 = { noinline nounwind ssp uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+fxsr,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { allocsize(0) "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+fxsr,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+fxsr,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { allocsize(0) }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"PIC Level", i32 2}
!1 = !{!"Apple LLVM version 9.0.0 (clang-900.0.39.2)"}
