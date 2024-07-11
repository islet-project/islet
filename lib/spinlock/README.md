## Usage of the library :
    
   use spinlock::spinlock
   use spinlock::Spinlock
   use spinlock::SpinlockGuard

## Locking mechanism :
   
   1. lock() - works as normal lock function
   2. lock_(RMI_ID) - takes RMI Interface Number of the interface holding the lock as argument
   
## Conditioanl compilation feature :
   
   add " --deadlock-test " in the command line argumrnt for Real Time Deadlock Test
   add " --deadlock-test --p-deadlock-test " in the command line for Potential Deadlock Test (runs with much overhead)

## APIS for potential deadlock analysis

   1. spinlock::checkdeadlock() - checks for potential deadlock with the best possible analysis
   2. spinlock::checkdeadlock_weak() - checks for potentail deadlock with weaker analysis 
      
      (use spinlock::spinlock must be included)