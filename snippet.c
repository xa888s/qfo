const size_t LAYERS = sizeof(keymaps) / sizeof(keymaps[0]);

enum result {
  OK,
  INVALID_INDEX,
};

void raw_hid_receive(uint8_t *data, uint8_t length) {
  uint8_t result = OK;

  // move layer if we got an index that is smaller than the amount of layers
  if (length > 0 && data[0] < LAYERS) {
    layer_move(data[0]);
  } else {
    result = INVALID_INDEX;
  }
  raw_hid_send(&result, length);
}
