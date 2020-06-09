#!/usr/bin/env perl
#

use strict;
use warnings;

my $device_start = 0x01C00000;
my $device_end   = 0x01F10000;
my $ram_start    = 0x40080000;
#my $ram_end      = 0x48000000;
my $ram_end      = 0x80000000;
my $page_table_size = 8192 * 8;
my $tt_firm_start = 0x2000000;

my %data = (
    device_start=>$device_start,
    device_end=>$device_end,
    ram_start=>$ram_start,
    ram_end=>$ram_end
    );


sub print_table{
    my ($name, $addr) = @_;

    print sprintf ("%20s:addr=%20X" ,$name, $addr);
    print ",l2 index=",$addr>>29 , "";
    print ",l3 index=",sprintf(" %06X ",($addr%(1<<28))>>16) , "\n";
}

sub init_l2_page_table{
    my ($tt, $l2_page_table, $l3_page_table, $start_addr, $end_addr)=@_;
    my $start = $start_addr>>29;
    my $end   = $end_addr>>29;
    for(my $i=$start;$i<=$end;$i++){
	printf("L2: %4d = %20X, %10X, %4d\n",
	       $i,
	       ($l3_page_table + $page_table_size * ($i-$start))
	       | 0b11
	       , $l2_page_table, $i);
	$tt->{$l2_page_table + $i * 8} = ($l3_page_table + $page_table_size * ($i-$start)) | 0b11
  }
}

sub init_l3_page_table{
    my ($tt, $l3_page_table, $l3_page_start_addr,
	$start_addr, $end_addr, $attr) = @_;

    my $l3_offset = ($l3_page_start_addr&(~((1<<29)-1)))>>16;
    printf("l3_offset=%10X\n", $l3_offset);
    my $start = $start_addr>>16;
    my $end   = $end_addr>>16;
    for(my $i=$start;$i<=$end;$i++){
	my $addr = $l3_page_table + ($i - $l3_offset) * 8;
	 printf("offset:%5d addr=%20X, %10X\n",
	        $i - $l3_offset,
	        $i <<16 
	        | $attr
	        | 0x3, $addr);
	$tt->{$addr} = $i <<16  | $attr | 0x3;
    }
}

foreach my $key (keys %data){
    print_table($key, $data{$key});
}

sub mmu_test {
    my ($tt, $sample_addr) = @_;
    my $l2_index = $sample_addr  >>29;
    my $l2_index_addr = $tt_firm_start + $l2_index * 8;
    if(exists $tt->{$l2_index_addr}){
	my $l3_page_table_addr = $tt->{$l2_index_addr} & (~0x3);
	my $l3_index = ($sample_addr % (1<<29)) >>16;
	my $l3_index_addr = $l3_page_table_addr + $l3_index * 8;
	if(exists $tt->{$l3_index_addr}){
	    my $r  = $tt->{$l3_index_addr};
	    my $pa       = $sample_addr & ((1<<16)-1);
	    my $result_addr = ($r  & (((1<<48) - 1) &(~0xffff))) + $pa;
	    printf ("aadr: %10X, %10X, r=%10X $l3_index=%4d pa=%06X\n", $sample_addr, $result_addr, $r, $l3_index, $pa);
	}
	else{
	    printf("error in l3 index, l2_index=%03d, l2_index_addr=%10X, l3_page_table_addr=%10X l3_index=%03d l3_index_addr=%10X\n",  $l2_index, $l2_index_addr, $l3_page_table_addr, $l3_index, $l3_index_addr);
	}
    }
    else{
	printf ("error in l2 index, %04d, %10X\n", $l2_index, $l2_index_addr);
    }
}

my %tt;

init_l2_page_table(\%tt,
		   $tt_firm_start, $tt_firm_start+$page_table_size,
		   $device_start, $device_end);
init_l3_page_table(\%tt,
		   $tt_firm_start+$page_table_size,
		   $device_start, $device_start, $device_end, 0);

init_l2_page_table(\%tt,
		   $tt_firm_start, $tt_firm_start+$page_table_size*2, $ram_start, $ram_end);

init_l3_page_table(\%tt,
		   $tt_firm_start+$page_table_size*2,
		   $ram_start, $ram_start, $ram_end, 0);

mmu_test(\%tt, $device_start + 1231);
mmu_test(\%tt, $ram_start + 12315);
