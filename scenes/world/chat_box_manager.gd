extends MessageHandler
class_name ChatBoxManager

signal new_message_sent(message:Message)

var messages:Array[Message]

class Message:
	var player:int
	var text:String


func send_message(message:String) -> void:
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	var player:int = (NetworkManager as NetNodeManager).get_id()
	send_message_final([player, message], [_get_value_type(null, 0), _get_value_type(player, 1)])

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 2
		1:
			return 6
	return -1

func _process_message(values: Array) -> void:
	var message:Message = Message.new()
	message.player = values[0]
	message.text = values[1]
	new_message_sent.emit(message)
	messages.append(message)
