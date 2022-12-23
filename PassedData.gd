extends Object

class_name PassedData

var Connected
var Dropped
var RawInput

var ConnectedAtStart
var Seed:int
var NextGame:String

func _init() -> void:
	
	Connected = []
	for n in 64:
		Connected.push_back(false)

	Dropped = []
	for n in 64:
		Dropped.push_back(false)

	RawInput = []
	for n in 64:
		RawInput.push_back(0)

	ConnectedAtStart = []
	for n in 64:
		ConnectedAtStart.push_back(false)

	Seed = 0
	NextGame = "None"

