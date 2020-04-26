//#include <stdint.h>
typedef unsigned long uint64_t;
typedef long int64_t;
typedef unsigned int uint32_t;
void uart0_putc(char c);
void uart0_puts(const char *s);
extern int64_t* __device_start;
extern int64_t* __device_end;
extern int64_t* __data_start;
extern int64_t* __data_end;
extern int64_t* __ram_start;
extern int64_t* __ram_start;
extern int64_t* __ram_end;
extern int64_t* __bss_start;
extern int64_t* __bss_end;

extern int64_t* __no_cache;
extern int64_t* __stack_start;
extern int64_t* __stack_firm_end;
extern int64_t* __stack_firm_start;
extern int64_t* __stack_el1_end;
extern int64_t* __stack_el1_start;
extern int64_t* __stack_el0_end;
extern int64_t* __stack_el0_start;
extern int64_t* __stack_end;

extern int64_t* __tt_firm_start;
extern int64_t* __tt_firm_end;
extern int64_t* __tt_el1_ttbr0_start;
extern int64_t* __tt_el1_ttbr0_end;
extern int64_t* __tt_el1_ttbr1_start;
extern int64_t* __tt_el1_ttbr1_end;
extern int64_t* __el0_heap_start;
extern int64_t* __el0_heap_end;
extern int64_t* _end;

#define PCR_CPU_X_BASE_ADDR		(0x01C20800)
#define PCR_PB_CFG0				(PCR_CPU_X_BASE_ADDR + (0x24 * 1))
#define PCR_PB_DAT				(PCR_CPU_X_BASE_ADDR + (0x24 * 1) + 0x10)
#define SUNXI_UART0_BASE	0x01C28000
#define UART0_FCR ((SUNXI_UART0_BASE) + 0x8)    /* fifo control register */
#define UART0_THR ((SUNXI_UART0_BASE) + 0x0)    /* transmit holding register */
#define UART0_LSR ((SUNXI_UART0_BASE) + 0x14)   /* line status register */

#define readl(addr)		(*((volatile unsigned long  *)(addr)))
#define writel(v, addr)		(*((volatile unsigned long  *)(addr)) = (unsigned long)(v))
#define readw(addr)		(*((volatile unsigned int *)(addr)))
#define writew(v, addr)		(*((volatile unsigned int *)(addr)) = (unsigned int)(v))
void uart0_init(){
  uint64_t * fcr_addr= UART0_FCR ;
  *fcr_addr = *fcr_addr | 1;
}

void uart0_putc(char c)
{
#if 0
  int i;
  for (i=0;i<10000;i++){
    asm volatile ("nop");
  }
#else
  //UART0_LSR[6] == 0 if UART0 FIFO buffer is full
  //UART0_LSR[5] == 0 if UART0 transmmision data is not empty
  //  while ((readl(UART0_LSR) & (3 << 5)) != (3<<5)) {}
  //  while ((readl(UART0_LSR) & (3 << 5)) != (3<<5)) {}
  while ((readw(UART0_LSR) & (1 << 5))==0) {}
#endif
  if(c== '\n'){
    writel('\r', UART0_THR);
  }
  writel(c, UART0_THR);
}

void uart0_puts(const char *s)
{
	while (*s) {
	  //		if (*s == '\n')
	  //			uart0_putc('\r');
		uart0_putc(*s++);
	}
}
void  uart0_hex(uint64_t h){
  int i;
  
  for (i=60; i>=0;i-=4){
    uint64_t n = (h>>i) & 0xF;
    n += n > 9 ? 0x37 : 0x30;
    uart0_putc(n);
  }
}

void  uart0_byte(uint64_t h){
  int i;
  
  uint64_t n = (h>>4) & 0xF;
  n += n > 9 ? 0x37 : 0x30;
  uart0_putc(n);
  n = h & 0xF;
  n += n > 9 ? 0x37 : 0x30;
  uart0_putc(n);

}
uint64_t get_device_start(void){
  uart0_puts("get device start\n");
  return (uint64_t)(__device_start);
}

uint64_t get_device_end(void){
  uart0_puts("get device end\n");
  return (uint64_t)(__device_end);
}

// 64KB page
// level 2 and 3 translation tables

const uint64_t PAGESIZE  = 64L * 1024;
// NSTable (63bit)
const uint64_t FLAG_L2_NS  = 1L << 63; // non secure table


const uint64_t FLAG_L3_XN    = 1L << 54; // execute never
const uint64_t FLAG_L3_PXN   = 1L << 53; // priviledged execute
const uint64_t FLAG_L3_CONT  = 1L << 52; // contiguous
const uint64_t FLAG_L3_DBM   = 1L << 51; // dirty bit modifier
const uint64_t FLAG_L3_AF    = 1L << 10; // access flag
const uint64_t FLAG_L3_NS    = 1L <<  5; // non secure

// [9:8]: Shareability attribute, for Normal memory
//    | Shareability
// ---|------------------
// 00 | non sharedable
// 01 | reserved
// 10 | outer sharedable
// 11 | inner sharedable
const uint64_t FLAG_L3_OSH  = 0b10 << 8;
const uint64_t FLAG_L3_ISH  = 0b11 << 8;

// [7:6]: access permissions
//    | Access from            |
//    | higher Exception level | Access from EL0
// ---|------------------------|-----------------
// 00 | read/write             | none
// 01 | read/write             | read/write
// 10 | read-only              | none
// 11 | read-only              | read-only
const uint64_t FLAG_L3_SH_RW_N   =    0;
const uint64_t FLAG_L3_SH_RW_RW  =    1 << 6;
const uint64_t FLAG_L3_SH_R_N    = 0b10 << 6;
const uint64_t FLAG_L3_SH_R_R    = 0b11 << 6;

// [4:2]: AttrIndx
// defined in MAIR register
const uint64_t FLAG_L3_ATTR_MEM  = 0     ; // normal memory
const uint64_t FLAG_L3_ATTR_DEV  = 1 << 2; // device MMIO
const uint64_t FLAG_L3_ATTR_NC   = 2 << 2; // non-cachable

static const int page_table_size = 8192 * 8;
/*
 *
 * Pine64の場合
 * 
 * 
 *                   M  K
 * device_start = 0x01C00000;
 * device_end   = 0x01F10000;
 * ram_start    = 0x40080000;
 * ram_end      = 0x48000000;

 * device_start = 0x01C0 0000;
 * device_end   = 0x01F1 0000;
 * ram_start    = 0x40080000;
 * ram_end      = 0x48000000;
 * 
 *   64k(Byte/page) * 8k(entry/L2 table) = 512k 
 */

static const void init_l2_page_table(uint64_t *l2_page_table,
				     uint64_t l3_page_table,
				     const uint64_t start_addr, const uint64_t end_addr){
  const uint64_t start = start_addr>>29;
  const uint64_t end =   end_addr>>29;
  uint64_t i;
  for(i=start;i<=end;i++){
    l2_page_table[i] =
      l3_page_table + page_table_size * (i-start)
      | 0b11;
  }
}


static const void init_l3_page_table(uint64_t *l3_page_table,
				     const uint64_t l3_page_start_addr,
				     const uint64_t start_addr, const uint64_t end_addr,
				     const uint64_t attr){

  uint64_t l3_offset = (l3_page_start_addr&(~((1<<29)-1)))>>16;
  uint64_t start = start_addr>>16;
  uint64_t end   = end_addr>>16;
  uint64_t i;
  
  for(i=start;i<end;i++){
    l3_page_table[i-l3_offset] = i <<16 | attr | 0b11;
  }
}

static const uint64_t device_start  = (uint64_t)(&__device_start);
static const uint64_t device_end    = (uint64_t)(&__device_end);
static const uint64_t data_start    = (uint64_t)(&__data_start);
static const uint64_t data_end      = (uint64_t)(&__data_end);
static const uint64_t ram_start     = (uint64_t)(&__ram_start);
static const uint64_t ram_end       = (uint64_t)(&__ram_end);
static const uint64_t bss_start     = (uint64_t)(&__bss_start);
static const uint64_t bss_end       = (uint64_t)(&__bss_end);
static const uint64_t no_cache      = (uint64_t)(&__no_cache);
static const uint64_t stack_start  = (uint64_t)(&__stack_start);
static const uint64_t stack_firm_end  = (uint64_t)(&__stack_firm_end);
static const uint64_t stack_firm_start  = (uint64_t)(&__stack_firm_start);
static const uint64_t stack_el1_end  = (uint64_t)(&__stack_el1_end);
static const uint64_t stack_el1_start  = (uint64_t)(&__stack_el1_start);
static const uint64_t stack_el0_end  = (uint64_t)(&__stack_el0_end);
static const uint64_t stack_el0_start  = (uint64_t)(&__stack_el0_start);
static const uint64_t stack_end  = (uint64_t)(&__stack_end);
static const uint64_t tt_firm_start  = (uint64_t)(&__tt_firm_start);
static const uint64_t tt_firm_end  = (uint64_t)(&__tt_firm_end);
static const uint64_t tt_el1_ttbr0_start  = (uint64_t)(&__tt_el1_ttbr0_start);
static const uint64_t tt_el1_ttbr0_end  = (uint64_t)(&__tt_el1_ttbr0_end);
static const uint64_t tt_el1_ttbr1_start  = (uint64_t)(&__tt_el1_ttbr1_start);
static const uint64_t tt_el1_ttbr1_end  = (uint64_t)(&__tt_el1_ttbr1_end);
static const uint64_t el0_heap_start  = (uint64_t)(&__el0_heap_start);
static const uint64_t el0_heap_end  = (uint64_t)(&__el0_heap_end);
static const uint64_t ttbr0         = (uint64_t)(&__tt_el1_ttbr0_start);
static const uint64_t ttbr1         = (uint64_t)(&__tt_el1_ttbr1_start);

uint64_t init_table_flat(uint64_t table_addr){
  // setup L2, L3 table for device
  // device_start = 0x01C0 0000;
  // device_end   = 0x01F1 0000;
  uint64_t *l2_table       = (uint64_t*)table_addr;
  uint64_t l3_table_top    = table_addr + page_table_size;
  uint64_t l3_device_table = l3_table_top;
  uint64_t l3_ram_table    = l3_table_top + page_table_size * 2;


  uint64_t *i;

  for(i=(uint64_t*)table_addr;(uint64_t)i<(uint64_t)table_addr+page_table_size*8;i++){
    *i = 0;
  }
  
  init_l2_page_table(l2_table,
		     l3_device_table,
		     device_start, device_end);
  

  init_l3_page_table((uint64_t*)l3_device_table,
		     device_start,
		     device_start, device_end,
   		     FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV);

  // ram_start    = 0x40080000;
  // ram_end      = 0x48000000;
  init_l2_page_table(l2_table,
		     l3_ram_table,
		     ram_start, ram_end);

  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     ram_start, ram_end,
		     FLAG_L3_NS | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM);

  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     ram_start, data_start,
		     FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_R_R | FLAG_L3_ATTR_MEM);

  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     data_start, no_cache,
		     FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM);


  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     no_cache, stack_start,
		     FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC);

  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     stack_start, stack_end,
		     FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM);

  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     stack_end, ram_end,
		     FLAG_L3_NS | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC);
  
  init_l3_page_table((uint64_t*)l3_ram_table,
		     ram_start,
		     tt_firm_start, ram_end,
		     FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC);
}


uint64_t get_mair() {
  return (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
    (0x44 << 16);   // AttrIdx=2: non cacheable
}

/// for TCR_EL2 and TCR_EL2
uint64_t get_tcr(){
  uint64_t mmfr;
  asm volatile("mrs %0, id_aa64mmfr0_el1" : "=r" (mmfr));
  uint64_t b = mmfr & 0xF;
  return (
	  1 << 31 | // Res1
	  1 << 23 | // Res1
	  b << 16 |
	  1 << 14 | // 64KiB granule
	  3 << 12 | // inner shadable
	  1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
	  1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
	  22        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
	  );
}

uint64_t update_sctlr(uint64_t sctlr){
    uint64_t new_sctlr =
        sctlr   |
        1L << 44 | // set DSSBS, enable speculative load and store
        1 << 12 | // set I, instruction cache
        1 <<  2 | // set C, data cache
        1;        // set M, enable MMU
    return (new_sctlr & (~(
			  1 << 25 | // clear EE
			  1 << 19 | // clear WXN
			  1 <<  3 | // clear SA
			  1 <<  1   // clear A
			   )));
      
}

/// mask firmware's stack and transition table
//fn mask_firm(tt: &'static mut [u64]) -> &'static mut [u64] {
void mask_firm(uint64_t table_addr, uint64_t table_top_addr){
  // do nothing
  // mask EL3 stack;
  uint64_t l3_offset = (table_top_addr&(~((1<<29)-1)))>>16;
  uint64_t* l3_page_table = (uint64_t*)(table_addr + page_table_size * 2);
  uint64_t start = tt_firm_start>>16;
  uint64_t end   = tt_firm_end>>16;
  uint64_t i;
  
  for(i=start;i<end;i++){
    l3_page_table[i-l3_offset] = 0;
  }
}
/// mask EL1's stack and transition table
void mask_el1(uint64_t addr, uint64_t table_top_addr){
  // do nothing
  // mask EL1, LE0, EL1 transition table
  //
  uint64_t i;
  uint64_t l3_offset = (table_top_addr&(~((1<<29)-1)))>>16;
  uint64_t* l3_page_table = (uint64_t*)(addr + page_table_size * 2);
  uint64_t start = stack_el0_start>>16;
  uint64_t end   = stack_el0_end>>16;

  for(i=start;i<end;i++){
    l3_page_table[i-l3_offset] = 0;
  }

  start = tt_el1_ttbr0_start>>16;
  end   = tt_el1_ttbr1_end>>16;
  for(i=start;i<end;i++){
    l3_page_table[i-l3_offset] = 0;
  }
}

void init_el1(){

  init_table_flat(ttbr0);
  //
  // mask el0 stack to detect stack over flow
  //
  init_table_flat(ttbr1);

  uint64_t mmfr;
  asm volatile("mrs %0, id_aa64mmfr0_el1" : "=r" (mmfr));
  uint64_t b = mmfr & 0xF;

  uint64_t tcr = 
         b << 32 |
         3 << 30 | // 64KiB granule, TTBR1_EL1
         3 << 28 | // inner shadable, TTBR1_EL1
         1 << 26 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
         1 << 24 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
        22 << 16 | // T1SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
         1 << 14 | // 64KiB granule
         3 << 12 | // inner shadable, TTBR0_EL1
         1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
         1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
        22;        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space

  // next, specify mapping characteristics in translate control register
  asm volatile("msr tcr_el1, %0" : : "r" (tcr));

  // tell the MMU where our translation tables are.
  asm volatile("msr ttbr0_el1, %0" : : "r" (ttbr0 + 1));
  asm volatile("msr ttbr1_el1, %0" : : "r" (ttbr1 + 1));

  // finally, toggle some bits in system control register to enable page translation
  uint64_t sctlr;
  asm volatile("dsb ish; isb; mrs %0, sctlr_el1" : "=r" (sctlr));
  sctlr = update_sctlr(sctlr);
  sctlr &= ~(
	     1 << 4 // clear SA0
	     );
  asm volatile("msr sctlr_el1, %0; dsb sy; isb" : : "r" (sctlr));
}
void dump_el3_table(void);
void init_el3(){
  init_table_flat(tt_firm_start);

  mask_el1(tt_firm_start, ram_start);

  // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
  asm volatile("msr mair_el3, %0" : : "r" (get_mair()));

  // next, specify mapping characteristics in translate control register
  asm volatile("msr tcr_el3, %0" : : "r" (get_tcr()));

    // tell the MMU where our translation tables are.
  //asm volatile("msr ttbr0_el3, %0" : : "r" (tt_firm_start + 1));
  asm volatile("msr ttbr0_el3, %0" : : "r" (tt_firm_start+1));

    // finally, toggle some bits in system control register to enable page translation
  uint64_t sctlr;
  asm volatile("dsb ish; isb; mrs %0, sctlr_el3" : "=r" (sctlr));
  // uart0_puts("init_el3:sctlr=");
  //  uart0_hex(sctlr);
  //  uart0_puts("\n");
  uint64_t new_sctlr = update_sctlr(sctlr);
  //  uart0_puts("init_el3:sctlr=");
  //  uart0_hex(new_sctlr);
  //  uart0_puts("\n");
  //  dump_el3_table();
  asm volatile("msr sctlr_el3, %0; dsb sy; isb" : : "r" (new_sctlr));
}

void enable_mmu(){
  uint32_t sctlr;
  asm volatile("mrs %0, SCTLR_EL3" : "=r"(sctlr)) ;
  uart0_puts("sctlr=");
  uart0_hex(sctlr);
  uart0_puts("\n");
}

void led_blink(void){
  unsigned long val = 0x04;
  while (1) {
    volatile int i;
    for(i = 0; i < 500000; i++);
    *(unsigned int *)(PCR_PB_DAT) ^= val;
  }
}

void dump_el3_table(){
  uint64_t *i ;
  for(i=(uint64_t*)tt_firm_start;(uint64_t)i<tt_firm_start+page_table_size*8;i++){
    if(*i !=0){
      uart0_hex((uint64_t)i);
      uart0_puts(": ");
      uart0_hex(*i);
      uart0_puts("\n");
    }
  }
}

//static const uint64_t device_start  = (uint64_t)(&__device_start);
//static const uint64_t device_end    = (uint64_t)(&__device_end);
//static const uint64_t ram_start     = (uint64_t)(&__ram_start);
//static const uint64_t stack_end     = (uint64_t)(&__stack_end);
//static const uint64_t ram_end       = (uint64_t)(&__ram_end);
//static const uint64_t tt_firm_start = (uint64_t)(&__tt_firm_start);
//static const uint64_t ttbr0         = (uint64_t)(&__tt_el1_ttbr0_start);
//static const uint64_t ttbr1         = (uint64_t)(&__tt_el1_ttbr1_start);


void mmu_init(){
  uint64_t * fcr_addr= UART0_FCR ;
  uart0_puts("UART_FCR=");
  uart0_hex(*fcr_addr);
  uart0_puts("\n");
  uart0_puts("mmu_init start\n");
  uart0_puts(  "device_start =");
  uart0_hex(device_start);
  uart0_puts("\ndevice_end   =");
  uart0_hex(device_end);
  uart0_puts("\nram_start    =");
  uart0_hex(ram_start);
  uart0_puts("\nstarck_end   =");
  uart0_hex(stack_end);
  uart0_puts("\ntt_firm_start=");
  uart0_hex(tt_firm_start);
  uart0_puts("\nttbr0        =");
  uart0_hex(ttbr0);
  uart0_puts("\nttbr1        =");
  uart0_hex(ttbr1);
  uart0_puts("\nram_end      =");
  uart0_hex(ram_end);
  
  uart0_puts("mmu_init el1 start\n");
  init_el1();
  uart0_puts("mmu_init el3 start\n");
  init_el3();
  //  dump_el3_table();
  ///  led_blink();
  enable_mmu();
  uart0_puts("mmu_init end\n");
}
