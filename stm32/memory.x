/* Memory layout for STM32H743ZI */
/* STM32H743ZI has 2MB Flash and 1MB RAM */

MEMORY
{
  /* Flash memory - where your program code lives */
  /* Reserve last 8KB for cryptographic keys (2048K - 8K = 2040K) */
  FLASH : ORIGIN = 0x08000000, LENGTH = 2040K
  
  /* Reserved flash section for Falcon512 keys */
  /* Located at end of flash: 0x08000000 + 2040K = 0x081FE000 */
  /* Size: 8KB (more than enough for SK=1281 bytes + PK=897 bytes = 2178 bytes) */
  KEYS : ORIGIN = 0x081FE000, LENGTH = 8K
  
  /* RAM - where your program's data and stack live */
  /* Note: Falcon512 requires significant stack space! */
  /* STM32H7 has multiple RAM regions, using DTCM RAM here */
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Stack size - Falcon512 operations need substantial stack */
/* Adjust based on your testing and available RAM */
_stack_size = 32K;

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE: Falcon512 signing uses recursion and needs significant stack! */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
