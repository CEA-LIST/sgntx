/*
    (C) Copyright 2017 CEA LIST. All Rights Reserved.
    Contributor(s): Thibaud Tortech & Sergiu Carpov

    This software is governed by the CeCILL-C license under French law and
    abiding by the rules of distribution of free software.  You can  use,
    modify and/ or redistribute the software under the terms of the CeCILL-C
    license as circulated by CEA, CNRS and INRIA at the following URL
    "http://www.cecill.info".

    As a counterpart to the access to the source code and  rights to copy,
    modify and redistribute granted by the license, users are provided only
    with a limited warranty  and the software's author,  the holder of the
    economic rights,  and the successive licensors  have only  limited
    liability.

    The fact that you are presently reading this means that you have had
    knowledge of the CeCILL-C license and that you accept its terms.
*/


#include <stdint.h>
#include <openssl/bio.h>
#include <openssl/evp.h>


void aes_gcm_encrypt(uint8_t* key, const uint8_t* pt_buff, uint64_t pt_size, uint8_t* ct_buff, uint8_t* iv, uint8_t* mac)
{
  unsigned int outlen;
  EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
  /* Set cipher type and mode */
  EVP_EncryptInit_ex(ctx, EVP_aes_128_gcm(), NULL, NULL, NULL);

  /* Set IV length if default 96 bits is not appropriate */
  EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);

  // printf("AES GCM Encrypt:\n");
  // printf("Plaintext %d:\n", 64);
  // BIO_dump_fp(stdout, pt_buff, 64);

  /* Initialise key and IV */
  EVP_EncryptInit_ex(ctx, NULL, NULL, key, iv);

  // /* Zero or more calls tol specify any AAD */
  // EVP_EncryptUpdate(ctx, NULL, &outlen, gcm_iv, sizeof(gcm_iv));

  /* Encrypt plaintext */
  EVP_EncryptUpdate(ctx, ct_buff, &outlen, pt_buff, pt_size);

  // /* Output encrypted block */
  // printf("CiphertextA %d:\n", outlen);
  // BIO_dump_fp(stdout, ct_buff, 64);

  /* Finalise: note get no output for GCM */
  EVP_EncryptFinal_ex(ctx, ct_buff, &outlen);

  // /* Output encrypted block */
  // printf("CiphertextB %d:\n", outlen);
  // BIO_dump_fp(stdout, ct_buff, 64);

  /* Get tag */
  EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, mac);

  // /* Output tag */
  // printf("Tag %d:\n", 16);
  // BIO_dump_fp(stdout, mac, 16);

  // /* Output tag */
  // printf("Buff %d:\n", 16+64);
  // BIO_dump_fp(stdout, iv, 16+64);

  EVP_CIPHER_CTX_free(ctx);
}
