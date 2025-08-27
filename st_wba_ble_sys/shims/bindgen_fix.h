#ifndef BINDGEN_FIX_H
#define BINDGEN_FIX_H

/* Baseline attributes */
#ifndef __packed
#define __packed __attribute__((__packed__))
#endif
#ifndef __PACKED
#define __PACKED __attribute__((__packed__))
#endif
#ifndef __weak
#define __weak __attribute__((weak))
#endif
#ifndef __WEAK
#define __WEAK __attribute__((weak))
#endif
#ifndef __IO
#define __IO volatile
#endif
#ifndef __ALIGN_BEGIN
#define __ALIGN_BEGIN
#endif
#ifndef __ALIGN_END
#define __ALIGN_END
#endif
#ifndef __ALIGNED
#define __ALIGNED(x) __attribute__((aligned(x)))
#endif
#ifndef ALIGN
#define ALIGN(x) __attribute__((aligned(x)))
#endif
#ifndef PLACE_IN_SECTION
#define PLACE_IN_SECTION(x) __attribute__((section(x)))
#endif

/* CMSIS-style convenience macros sometimes used by ST headers */
#ifndef __INLINE
#define __INLINE inline
#endif
#ifndef __STATIC_INLINE
#define __STATIC_INLINE static inline
#endif
#ifndef __STATIC_FORCEINLINE
#define __STATIC_FORCEINLINE __attribute__((always_inline)) static inline
#endif
#ifndef __NO_RETURN
#define __NO_RETURN __attribute__((noreturn))
#endif
#ifndef __UNUSED
#define __UNUSED __attribute__((unused))
#endif
#ifndef __USED
#define __USED __attribute__((used))
#endif
#ifndef __ALWAYS_INLINE
#define __ALWAYS_INLINE __attribute__((always_inline))
#endif

/* Token-style packed pairs some ST drops use */
#ifndef __PACKED_BEGIN
#define __PACKED_BEGIN
#endif
#ifndef __PACKED_END
#define __PACKED_END __attribute__((__packed__))
#endif

/* Token-style struct/union spellings (typedef PACKED_STRUCT {..} name;) */
#ifndef __PACKED_STRUCT
#define __PACKED_STRUCT struct __attribute__((__packed__))
#endif
#ifndef __PACKED_UNION
#define __PACKED_UNION  union  __attribute__((__packed__))
#endif
#ifndef PACKED_STRUCT
#define PACKED_STRUCT struct __attribute__((__packed__))
#endif
#ifndef PACKED_UNION
#define PACKED_UNION  union  __attribute__((__packed__))
#endif

/* Function-like helpers used in some generators (typedef PACKED_STRUCT(TypeDecl);) */
#ifndef PACKED
#define PACKED __attribute__((__packed__)) /* token-style: PACKED struct {..} name; */
#endif
#ifndef PACKED_STRUCT_DECL
#define PACKED_STRUCT_DECL(x) x __attribute__((__packed__))
#endif
#ifndef PACKED_UNION_DECL
#define PACKED_UNION_DECL(x) x __attribute__((__packed__))
#endif

#endif /* BINDGEN_FIX_H */
