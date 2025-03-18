#include "../picojpeg/picojpeg.h"
#include "libremu.h"
#include <stddef.h>

typedef unsigned char uint8;
typedef unsigned int uint;

unsigned char pjpeg_need_bytes_callback(unsigned char* pBuf, unsigned char buf_size, unsigned char *pBytes_actually_read, void *pCallback_data);

int main() {
    char status;
    int decoded_width, decoded_height, row_pitch;
    int mcu_x = 0;
    int mcu_y = 0;
    int x,y;
    int to_write;
    int offset = 0;
    unsigned int row_blocks_per_mcu, col_blocks_per_mcu;
    int bonus_offset = 0;
    pjpeg_image_info_t image_info;
    status = pjpeg_decode_init(&image_info, pjpeg_need_bytes_callback, NULL, 0);

    if (status)
    {
        outs("pjpeg_decode_init() failed");
        if (status == PJPG_UNSUPPORTED_MODE)
        {
            outs("Progressive JPEG files are not supported.\r\n");
        }

        return status;
    }
    decoded_width = image_info.m_width;
    decoded_height = image_info.m_height;

    row_pitch = decoded_width * image_info.m_comps;
    row_blocks_per_mcu = image_info.m_MCUWidth >> 3;
    col_blocks_per_mcu = image_info.m_MCUHeight >> 3;

    while(1) {
        status = pjpeg_decode_mcu();

        if (status)
        {
            if (status != PJPG_NO_MORE_BLOCKS)
            {
                outs("pjpeg_decode_mcu() failed\r\n");

                return 1;
            }

            // Sholud mean no more blocks here
            break;
        }


        //copy data to framebuffer
        for (y = 0; y < image_info.m_MCUHeight; y += 8)
        {

            for (x = 0; x < image_info.m_MCUWidth; x += 8)
            {
                bonus_offset = (x * 8)  + (y * 16);
                int bx, by;
                for (by = 0; by < 8; by++)
                {

                    for (bx = 0; bx < 8; bx++) {
                        to_write = image_info.m_pMCUBufR[bx+8*by + bonus_offset];
                        to_write = to_write << 8;
                        to_write |= image_info.m_pMCUBufG[bx+8*by + bonus_offset];
                        to_write = to_write << 8;
                        to_write |= image_info.m_pMCUBufB[bx+8*by + bonus_offset];
                        FRAMEBUFFER[offset+(by+y)*WIDTH+bx+x] = to_write;
                    }
                }
                //to_write = to_write << 8;

                // Where is the color bruh
            }

        }
        offset += 16;
        if (offset % WIDTH == 0) {
            // We added one row, so add the rest
            offset += (image_info.m_MCUHeight-1) * WIDTH;
        }
    }
    while(1) {}
    return 0;

}
