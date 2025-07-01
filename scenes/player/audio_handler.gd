extends Node

var effect:AudioEffectCapture
var is_recording:bool = false

func _ready() -> void:
	#AudioServer.input_device = AudioServer.get_input_device_list()[1] 
	# We get the index of the "Record" bus.
	var idx:int = AudioServer.get_bus_index("Record")
	# And use it to retrieve its first effect, which has been defined
	# as an "AudioEffectRecord" resource.
	effect = AudioServer.get_bus_effect(idx, 0)

func _physics_process(_delta: float) -> void:
	while true:
		var tmp:PackedVector2Array = effect.get_buffer(960)
		if tmp.size() == 0:
			break
		(NetworkManager as NetNodeManager).transmit_audio(tmp)
	((get_child(1) as AudioStreamPlayer).stream as VoiceStream).get_current_playback().buffer_audio((NetworkManager as NetNodeManager).get_audio())
