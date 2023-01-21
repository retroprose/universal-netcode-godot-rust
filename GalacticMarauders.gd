extends Node2D


var SyncedMask:BitMask = BitMask.new(55, 56);
var XMask:BitMask = BitMask.new(0, 16);
var LeftMask:BitMask = BitMask.new(16, 17);
var RightMask:BitMask = BitMask.new(17, 18);
var PrimaryMask:BitMask = BitMask.new(18, 19);


var MousePosition
var KeyLeft:bool
var KeyRight:bool
var KeySpace:bool

var localSlot:int
var passed
var rust

func set_input():
	for n in 64:
		rust.custom_set_input(n, passed.Connected[n], passed.ConnectedAtStart[n], passed.Dropped[n], passed.RawInput[n])

func init(l, p):
	localSlot = l
	passed = p

	rust = get_node("Rust")
	set_input()
	rust.custom_init(localSlot, passed.Seed)
	
	KeyLeft = false
	KeyRight = false
	KeySpace = false
	passed.NextGame = "None"


func main_update():
	set_input()
	rust.custom_update()
	if rust.custom_game_over() == true:
		passed.NextGame = "Intermission"


func copy_state():
	rust.custom_copy()
	pass
	
func fast_forward():
	set_input()
	rust.custom_fast_forward()
	pass


func process_input() -> int:
	var ret:int = 0

	var X:int = MousePosition.x

	var Left:int = 0
	var Right:int = 0
	var Primary:int = 0

	if KeyLeft == true:
		Left = 1
	if KeyRight == true:
		Right = 1
	if KeySpace == true:
		Primary = 1

	ret |= XMask.Encode(X)
	ret |= PrimaryMask.Encode(Primary)
	ret |= LeftMask.Encode(Left)
	ret |= RightMask.Encode(Right)

	ret |= SyncedMask.Encode(1)

	return ret;




func process_output(display:Object):
	
	rust.custom_render(display)
	
	#display.clear_sprites()
	
	#for n in 500:
	#	display.create_invader(
	#			rng.randi_range(0, 49), 
	#			rng.randf_range(-480.0, 480.0), 
	#			rng.randf_range(-270.0, 270.0),
	#			2.0,
	#			2.0
	#			)



func _input(event):
	#rint(event.as_text())	
	if event is InputEventMouseMotion:
		MousePosition = to_local(event.position)
	
	if event is InputEventKey:
		if event.pressed:
			if event.scancode == KEY_LEFT:
				KeyLeft = true
			if event.scancode == KEY_RIGHT:
				KeyRight = true
			if event.scancode == KEY_SPACE:
				KeySpace = true

		if not event.pressed:
			if event.scancode == KEY_LEFT:
				KeyLeft = false
			if event.scancode == KEY_RIGHT:
				KeyRight = false
			if event.scancode == KEY_SPACE:
				KeySpace = false






# Called when the node enters the scene tree for the first time.
#func _ready():
#	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
