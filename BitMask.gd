extends Object

class_name BitMask

	  
var mask:int
var shift:int

func _init(start, end) -> void:
	# cap size
	if end >= 64:
		end = 63
		
	mask = 0x0000000000000000
	shift = start
	for i in range(start, end):
		mask |= (1 << i)


func Decode(value:int) -> int:
	return (value & mask) >> shift


func Encode(value:int) -> int:
	return (value << shift) & mask

