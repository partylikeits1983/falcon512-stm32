/* Memory layout for STM32F407 (adjust for your specific chip) */
/* STM32F407VG has 1MB Flash and 192KB RAM (128KB + 64KB CCM) */

MEMORY
{
  /* Flash memory - where your program code lives */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  
  /* RAM - where your program's data and stack live */
  /* Note: Falcon512 requires significant stack space! */
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Stack size - Falcon512 operations need substantial stack */
/* Adjust based on your testing and available RAM */
_stack_size = 32K;

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE: Falcon512 signing uses recursion and needs significant stack! */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
