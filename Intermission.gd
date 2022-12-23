extends Node2D


var SyncedMask:BitMask = BitMask.new(55, 56);
var XMask:BitMask = BitMask.new(0, 16);
var YMask:BitMask = BitMask.new(16, 32);
var SeedMask:BitMask = BitMask.new(32, 48);
var ReadyMask:BitMask = BitMask.new(48, 49);

var ready_data = null
var ready_map = null
var ready_width:int

var rng = RandomNumberGenerator.new()

var localSlot
var passed
var main = null

var mousePosition = null

var playerPosition
var playerReady

var allSynced:bool
var allReady:bool


func should_start() -> bool:
	var inGame:int
	var ready:int
	var start:bool = true
	for i in range(0, passed.Connected.size()):
		if passed.Connected[i] == true:
			inGame = SyncedMask.Decode(passed.RawInput[i])
			ready = ReadyMask.Decode(passed.RawInput[i])
			if inGame == 1 || ready == 0:
				start = false
	return start

func should_end() -> bool:
	var inGame:int
	var start:bool = true
	for i in range(0, passed.Connected.size()):
		if passed.ConnectedAtStart[i] == true && passed.Connected[i] == true:
			inGame = SyncedMask.Decode(passed.RawInput[i])
			if inGame == 0:
				start = false;
	return start


func set_ready_at_start():
	for i in range(0, passed.Connected.size()):
		passed.ConnectedAtStart[i] = passed.Connected[i]


func get_seed() -> int:
	var rnd:int = 0
	for i in range(0, passed.Connected.size()):
		if passed.Connected[i] == true:
			rnd = SeedMask.Decode(passed.RawInput[i])
	return rnd



func init(l, p):
	localSlot = l
	passed = p
	
	playerReady = false
	playerPosition = Vector2(0.0, -1080.0 / 4);
	allSynced = false;
	allReady = false;
	
	passed.NextGame = "None"
	passed.Seed = 0
	for i in range(0, passed.ConnectedAtStart.size()):
		passed.ConnectedAtStart[i] = false


func main_update():
	if allReady == false:
		if should_start() == true:
			passed.Seed = get_seed()
			set_ready_at_start()
			allReady = true
	else:
		if should_end() == true:
			passed.NextGame = "GalacticMarauders"
			allSynced = true


func copy_state():
	pass
	
func fast_forward():
	pass


func process_input() -> int:
	
	var ret:int = 0

	if allReady == true:
		ret |= SyncedMask.Encode(1)
		return ret

	var mouse = mousePosition
	
	mouse.y = -mouse.y

	if mouse.x > 960.0 || mouse.x < -960.0 || mouse.y > 540.0 || mouse.y < -540.0:
		# don't do anything
		pass
	else:
		playerPosition += (mouse - playerPosition).normalized() * 4.0
	

	if playerPosition.x >= 536.0:
		playerPosition.x = 536.0
	if playerPosition.x <= -536.0:
		playerPosition.x = -536.0
	if playerPosition.y >= 536.0:
		playerPosition.x = 536.0
	if playerPosition.y <= -536.0:
		playerPosition.x = -536.0

	var X:int = playerPosition.x
	var Y:int = playerPosition.y

	playerReady = false
	var intReady:int = 0
	if Y > 0:
		playerReady = true
		intReady = 1

	# convert X and Y into short signed integer

	var rnd:int = rng.randi_range(0, 0xffff)

	ret |= XMask.Encode(X)
	ret |= YMask.Encode(Y)
	ret |= ReadyMask.Encode(intReady)
	ret |= SeedMask.Encode(rnd)

	return ret;



func process_output(display:Object):
	
	display.clear_sprites()
	
	var t:int
	var i:int
	var j:int
	var r:int
	var c:int
	
	for k in ready_map.size():
		t = ready_map[k]
		i = k % ready_width
		j = k / ready_width
		r = t % 256
		c = t / 256
		# also if all ready
		if c == 4 && playerReady == true:
			c = 12
		#display.create_ascii(r * c, i * 32.0 + 16.0, j * 32.0 + 16.0, 4.0, 4.0)
		display.create_ascii(c * 256 + r, i * 32.0 + 16.0 - 960.0 + 416.0, j * 32.0 + 16.0 - 540.0, 4.0, 4.0)

	var inGame:int
	var status:String
	var x:float = -900.0
	var y:float = 500.0
	var xpos:float
	var ypos:float
	
	for n in range(0, passed.RawInput.size()):
		status = "---------"
		inGame = SyncedMask.Decode(passed.RawInput[n])
		if passed.Connected[n] == true && inGame == 0:
			status = "In Lobby"
			i = XMask.Decode(passed.RawInput[n])
			j = YMask.Decode(passed.RawInput[n])
			if i > 32767:
				i = i - 0x10000
			if j > 32767:
				j = j - 0x10000
			
			xpos = float(i)
			ypos = float(j)
			
			t = 1
			if n == localSlot:
				t = 2
			display.create_ascii(t + 256 * 15, xpos, -ypos, 4.0, 4.0)

		if passed.Connected[n] == true && inGame == 1:
			status = "In Game"
	
		display.create_text(x, -y, "Slot %d: %s" % [n, status])
		y -= 16.0
	



func _input(event):
	if event is InputEventMouseMotion:
		mousePosition = to_local(event.position)
		#print("Mouse Motion at: ", mousePosition)





# Called when the node enters the scene tree for the first time.
func _ready():
	var file = File.new()
	file.open("res://sprites/ready.json", File.READ)
	var content = file.get_as_text()
	file.close()
	ready_data = JSON.parse(content).result

	var obj = ready_data

	var width:int = obj["width"]
	var height:int = obj["height"]
	var t:int

	ready_width = width
	ready_map = []
	for i in range(0, width * height):
		ready_map.push_back(0)

	for j in range(0, height):
		for i in range(0, width):
			#t = obj["layers"][0]["data"][(height - j - 1) * width + i]
			t = obj["layers"][0]["data"][j * width + i]
			ready_map[j * width + i] = t - 1
	
	
	
# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
