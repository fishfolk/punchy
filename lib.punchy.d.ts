declare namespace punchy {
    interface ItemGrabEvent {
        fighter: any,
    }

    function getItemGrabEvents(): ItemGrabEvent[]
}