#include "render_packet.h"

#include "config.h"

#include "../lib/utlist.h"
#include <stdlib.h>

senie_render_packet* render_packet_allocate(i32 max_vertices,
                                            i32 vbuf_element_size,
                                            i32 cbuf_element_size,
                                            i32 tbuf_element_size) {
  senie_render_packet* render_packet = (senie_render_packet*)calloc(1, sizeof(senie_render_packet));

  render_packet->num_vertices = 0;

  render_packet->vbuf = (f32*)calloc(max_vertices * vbuf_element_size, sizeof(f32));
  render_packet->cbuf = (f32*)calloc(max_vertices * cbuf_element_size, sizeof(f32));
  render_packet->tbuf = (f32*)calloc(max_vertices * tbuf_element_size, sizeof(f32));

  render_packet->prev = NULL;
  render_packet->next = NULL;

  return render_packet;
}

void render_packet_free(senie_render_packet* render_packet) {
  free(render_packet->vbuf);
  free(render_packet->cbuf);
  free(render_packet->tbuf);

  free(render_packet);
}

senie_render_data* render_data_allocate(i32 max_vertices) {
  senie_render_data* render_data = (senie_render_data*)calloc(1, sizeof(senie_render_data));

  render_data->max_vertices          = max_vertices;
  render_data->num_render_packets    = 0;
  render_data->render_packets        = NULL;
  render_data->current_render_packet = NULL;

  render_data->vbuf_element_size = 2;
  render_data->cbuf_element_size = 4;
  render_data->tbuf_element_size = 2;

  return render_data;
}

void render_data_free_render_packets(senie_render_data* render_data) {
  if (render_data == NULL) {
    return;
  }

  senie_render_packet* render_packet = render_data->render_packets;
  senie_render_packet* next          = NULL;

  while (render_packet != NULL) {
    next = render_packet->next;
    render_packet_free(render_packet);
    render_packet = next;
  }

  render_data->num_render_packets    = 0;
  render_data->render_packets        = NULL;
  render_data->current_render_packet = NULL;
}

void render_data_free(senie_render_data* render_data) {
  if (render_data == NULL) {
    return;
  }

  render_data_free_render_packets(render_data);

  free(render_data);
}

senie_render_packet* add_render_packet(senie_render_data* render_data) {
  RETURN_IF_NULL(render_data, "add_render_packet: render_data is a NULL pointer");

  senie_render_packet* render_packet = render_packet_allocate(render_data->max_vertices,
                                                              render_data->vbuf_element_size,
                                                              render_data->cbuf_element_size,
                                                              render_data->tbuf_element_size);
  DL_APPEND(render_data->render_packets, render_packet);
  render_data->current_render_packet = render_packet;

  render_data->num_render_packets++;

  return render_packet;
}

senie_render_packet* get_render_packet(senie_render_data* render_data, i32 index) {
  i32                  i             = 0;
  senie_render_packet* render_packet = render_data->render_packets;

  while (render_packet != NULL) {
    if (i == index) {
      return render_packet;
    }
    i++;
    render_packet = render_packet->next;
  }

  return NULL;
}