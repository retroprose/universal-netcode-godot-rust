extends Node2D


# state interface:
# running
# current_state
# init
# process_input
# process_output
# stop


#var rng = RandomNumberGenerator.new()

#var websocket_url = "wss://fast-forward-64.herokuapp.com/session/0"
#var websocket_url = "ws://localhost:8080/"
var websocket_url = "wss://go-gin-web-server-9trz.onrender.com/session/0"
var MaxForward = 4
var MaxDelay = 1


var _client = WebSocketClient.new()

enum {  Null = 0,
		WaitingOnRoomSelect,
		WaitingToConnect,
		CreatingFirst,
		WaitingOnFirst,
		Running }

var StateMap
var InputQueue
var UploadQueue
var State
var RunningGame
var Passed
var GameInputUpload
var LastPopped

var LocalSlot:int
var RoomSelect:int

var DeltaTime:float
var WaitTime:float

var FrameSkip:int
var NonEmptyFrames:int
var EmptyFrames:int

func send_input(input:int):
	# convert GameInput to bytes
	GameInputUpload[0] = (input & 0x00000000000000ff) >> (0 * 8)
	GameInputUpload[1] = (input & 0x000000000000ff00) >> (1 * 8)
	GameInputUpload[2] = (input & 0x0000000000ff0000) >> (2 * 8)
	GameInputUpload[3] = (input & 0x00000000ff000000) >> (3 * 8)
	GameInputUpload[4] = (input & 0x000000ff00000000) >> (4 * 8)
	GameInputUpload[5] = (input & 0x0000ff0000000000) >> (5 * 8)
	GameInputUpload[6] = (input & 0x00ff000000000000) >> (6 * 8)
	_client.get_peer(1).put_packet(GameInputUpload)


# read in little endian I think...
func read_passed() -> bool:
	if InputQueue.size() == 0:
		return false

	LastPopped = InputQueue.pop_front()

	var offset:int
	var active:int
	var broken:int
	var raw:int
	
	offset = 0
	active = 0
	for i in 8:
		active |= LastPopped[offset + i] << (i * 8)
		
	offset = 8
	broken = 0
	for i in 8:
		broken |= LastPopped[offset + i] << (i * 8)
	
	for n in Passed.RawInput.size():
		Passed.Connected[n] = active & (0x1 << n) != 0x0
		Passed.Dropped[n] = broken & (0x1 << n) != 0x0
		
		offset = n * 7 + 16
		raw = 0
		for i in 7:
			raw |= LastPopped[offset + i] << (i * 8)
		
		Passed.RawInput[n] = raw

	return true




# Called when the node enters the scene tree for the first time.
func _ready():
		
	StateMap = {
		"None": null,
		"Intermission": get_node("States").get_node("Intermission"),
		"GalacticMarauders": get_node("States").get_node("GalacticMarauders")
	}
	
	LocalSlot = -1

	InputQueue = []
	UploadQueue = []

	State = WaitingOnRoomSelect
	
	DeltaTime = 0.0
	WaitTime = 0.0

	EmptyFrames = 0
	NonEmptyFrames = 0

	RoomSelect = -1

	Passed = PassedData.new()
	GameInputUpload = PoolByteArray()
	for n in 7:
		GameInputUpload.append(0)
		
		

func _input(event):
	#rint(event.as_text())	
	if event is InputEventKey:
		if event.pressed:
			if event.scancode == KEY_0:
				RoomSelect = 0
			if event.scancode == KEY_1:
				RoomSelect = 1
			if event.scancode == KEY_2:
				RoomSelect = 2
			if event.scancode == KEY_3:
				RoomSelect = 3
			if event.scancode == KEY_4:
				RoomSelect = 4
			if event.scancode == KEY_5:
				RoomSelect = 5
			if event.scancode == KEY_6:
				RoomSelect = 6
			if event.scancode == KEY_7:
				RoomSelect = 7
			if event.scancode == KEY_8:
				RoomSelect = 8
			if event.scancode == KEY_9:
				RoomSelect = 9


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):


	_client.poll()

	match (State):
		WaitingOnRoomSelect:
			if RoomSelect != -1:
				print("Joining ", RoomSelect)
				join(RoomSelect)
				State = WaitingToConnect
			
		WaitingToConnect:
			if InputQueue.size() > 0 && InputQueue.front().size() == 1:
				LocalSlot = InputQueue.front()[0]
				InputQueue.pop_front()
				State = CreatingFirst
				print("Assigned: ", LocalSlot)
		
		CreatingFirst:
			print("CreatingFirst")
			RunningGame = StateMap["Intermission"]
			RunningGame.init(LocalSlot, Passed)
			State = WaitingOnFirst
						
		WaitingOnFirst:
			print("WaitingOnFirst")
			var GameInput:int = RunningGame.process_input()
			
			WaitTime += delta
			if WaitTime > 0.033:
				WaitTime -= 0.033
				if UploadQueue.size() < MaxForward:
					UploadQueue.push_back(GameInput)
					send_input(GameInput)
				
			var done:bool = false
			while !done:
				if InputQueue.size() == 0:
					done = true
				else:
					read_passed()
					if Passed.Dropped[LocalSlot] == false:
						InputQueue.push_front(LastPopped)
						State = Running
						done = true
			
		Running:
			#print("Running")
			var GameInput:int = RunningGame.process_input()
			
			WaitTime += delta
			if WaitTime > 0.033:
				WaitTime -= 0.033
				if FrameSkip > 0:
					FrameSkip -= 1
				else:
					if UploadQueue.size() < MaxForward:
						UploadQueue.push_back(GameInput)
						send_input(GameInput)
			
			
			var frameCount:int = 0
			var dropped:int = 0
			while InputQueue.size() > 0:
				read_passed()
				
				if Passed.Dropped[LocalSlot] == false:
					UploadQueue.pop_front()
					NonEmptyFrames += 1
				else:
					FrameSkip += 1
					dropped += 1
					EmptyFrames += 1
				frameCount += 1
				
				RunningGame.main_update()
				if Passed.NextGame != "None":
					RunningGame = StateMap[Passed.NextGame]
					RunningGame.init(LocalSlot, Passed)

				# copy subset of gamestate for the fast forward 
				RunningGame.copy_state()

				# fast forward until...

				#Debug.Log("--------------------");
				var frameCounter:int = 0
				for ui in UploadQueue:
					if frameCounter < UploadQueue.size() - MaxDelay:
						#Debug.Log($"{ui.frame}");
						# insert next player input
						Passed.RawInput[LocalSlot] = ui;
						Passed.Dropped[LocalSlot] = false;
						RunningGame.fast_forward()
					frameCounter += 1;
				

				RunningGame.process_output( get_node("Display") )
				
		_:
			# default, do nothing
			pass



# I think all of these functions work...
func join(room):
	
	 # Connect base signals to get notified of connection open, close, and errors.
	_client.connect("connection_closed", self, "_closed")
	_client.connect("connection_error", self, "_closed")
	_client.connect("connection_established", self, "_connected")
	# This signal is emitted when not using the Multiplayer API every time
	# a full packet is received.
	# Alternatively, you could check get_peer(1).get_available_packets() in a loop.
	_client.connect("data_received", self, "_on_data")

	# Initiate connection to the given URL.
	#var err = _client.connect_to_url(websocket_url, ["lws-mirror-protocol"])
	var err = _client.connect_to_url(websocket_url)
	if err != OK:
		print("Unable to connect")
		set_process(false)


func _closed(was_clean = false):
	# was_clean will tell you if the disconnection was correctly notified
	# by the remote peer before closing the socket.
	print("Closed, clean: ", was_clean)
	set_process(false)

func _connected(proto = ""):
	# This is called on connection, "proto" will be the selected WebSocket
	# sub-protocol (which is optional)
	print("Connected with protocol: ", proto)
	# You MUST always use get_peer(1).put_packet to send data to server,
	# and not put_packet directly when not using the MultiplayerAPI.
	#_client.get_peer(1).put_packet("Test packet".to_utf8())

func _on_data():
	InputQueue.push_back(_client.get_peer(1).get_packet())
	
		


