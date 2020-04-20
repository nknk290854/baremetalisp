#include <stdint.h>

extern int64_t* __device_start;
extern int64_t* __device_end;
extern int64_t* __data_start;
extern int64_t* __data_end;
extern int64_t* __ram_start;
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
extern int64_t* __ram_end;

#define SUNXI_UART0_BASE	0x01C28000
#define UART0_THR ((SUNXI_UART0_BASE) + 0x0)    /* transmit holding register */
#define UART0_LSR ((SUNXI_UART0_BASE) + 0x14)   /* line status register */

#define readl(addr)		(*((volatile unsigned long  *)(addr)))
#define writel(v, addr)		(*((volatile unsigned long  *)(addr)) = (unsigned long)(v))
static void uart_putc_for_c(char c)
{
  	while (!(readl(UART0_LSR) & (1 << 6))) {}
	writel(c, UART0_THR);
}

static void uart_puts_for_c(const char *s)
{
	while (*s) {
		if (*s == '\n')
			uart_putc_for_c('\r');
		uart_putc_for_c(*s++);
	}
}


uint64_t get_device_start(void){
  uart_puts_for_c("get device start\n");
  return (uint64_t)(&__device_start);
}

uint64_t get_device_end(void){
    uart_puts_for_c("get device end\n");
  return (uint64_t)(&__device_end);
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
				     uint64_t *l3_page_table,
				     const uint64_t start_addr, const uint64_t end_addr){
  const uint64_t start = start_addr>>29;
  const uint64_t end =   end_addr>>29;
  uint64_t i;
  for(i=start;i<=end;i++){
    l2_page_table[i] =
      ((uint64_t) l3_page_table) + page_table_size * (i-start)
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
  
  for(i=start;i<=end;i++){
    l3_page_table[i-l3_offset] = i <<16 | attr | 0b11;
  }
}

uint64_t init_table_flat(uint64_t table_addr){
  const uint64_t device_start  = (uint64_t)(&__device_start);
  const uint64_t device_end    = (uint64_t)(&__device_end);
  const uint64_t ram_start     = (uint64_t)(&__ram_start);
  const uint64_t ram_end       = (uint64_t)(&__ram_end);
  const uint64_t el3_table_top = (uint64_t)(&__tt_firm_start);
  // setup L2, L3 table for device
  // device_start = 0x01C0 0000;
  // device_end   = 0x01F1 0000;
  uint64_t *el3_l2_table        = (uint64_t *)(el3_table_top);
  uint64_t *el3_l3_device_table = (uint64_t *)(el3_table_top + page_table_size);
  uint64_t *el3_l3_ram_table    = (uint64_t *)(el3_table_top + page_table_size * 2);
  
  init_l2_page_table(el3_l2_table,
		     el3_l3_device_table,
		     device_start, device_end);
  

  init_l3_page_table(el3_l3_device_table,
		     device_start,
		     device_start, device_end,
		     FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV);

  // ram_start    = 0x40080000;
  // ram_end      = 0x48000000;
  init_l2_page_table(el3_l2_table,
		     el3_l3_ram_table,
		     ram_start, ram_end);

  init_l3_page_table(el3_l3_ram_table,
		     ram_start,
		     ram_start, ram_end,
		     FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM);
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
    return (new_sctlr & !(
			  1 << 25 | // clear EE
			  1 << 19 | // clear WXN
			  1 <<  3 | // clear SA
			  1 <<  1   // clear A
			  ));
      
}

/// mask firmware's stack and transition table
//fn mask_firm(tt: &'static mut [u64]) -> &'static mut [u64] {
void mask_firm(uint64_t addr){
  // do nothing
  // mask EL3 stack;
  //
}
/// mask EL1's stack and transition table
void mask_el1(){
  // do nothing
  // mask EL1, LE0, EL1 transition table
  //
}

void init_el1(){

  uint64_t ttbr0 = (uint64_t)(&__tt_el1_ttbr0_start);
  init_table_flat(ttbr0);

  //
  // mask el0 stack to detect stack over flow
  //
  uint64_t ttbr1 = (uint64_t)(&__tt_el1_ttbr1_start);
  init_table_flat(ttbr1);


  return;
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
  sctlr &= !(
	     1 << 4 // clear SA0
	     );
  asm volatile("msr sctlr_el1, %0; dsb sy; isb" : : "r" (sctlr));
}

void init_el3(){
  uint64_t tt_firm_start = (uint64_t)(&__tt_firm_start);
  init_table_flat(tt_firm_start);

  mask_el1();

  return;
  // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
  asm volatile("msr mair_el3, %0" : : "r" (get_mair()));

  // next, specify mapping characteristics in translate control register
  asm volatile("msr tcr_el3, %0" : : "r" (get_tcr()));

    // tell the MMU where our translation tables are.
  asm volatile("msr ttbr0_el3, %0" : : "r" (tt_firm_start + 1));

    // finally, toggle some bits in system control register to enable page translation
  uint64_t sctlr;
  asm volatile("dsb ish; isb; mrs %0, sctlr_el3" : "=r" (sctlr));
  sctlr = update_sctlr(sctlr);
  asm volatile("msr sctlr_el3, %0; dsb sy; isb" : : "r" (sctlr));
}

void mmu_init(){
  uart_puts_for_c("mmu_init start\n");
  uart_puts_for_c("mmu_init el1 start\n");
  //  init_el1();
  uart_puts_for_c("mmu_init el3 start\n");
  //  init_el3();
  uart_puts_for_c("mmu_init end\n");
}
