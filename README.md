# wp_encrypt_crack

An encryption cracker for Word Perfect 4.2 (Originally released 1987). 

This program attempts to recover encryption keys for documents encrypted with Word Perfect 4.2. It has been tested to work with english based documents, written in Ascii. 

The tool came about because a couple of co-workers were discussing old files and one had an old word perfect document he wanted to try to decrypt. The other co-worker thought this would be a fun challenge, and this is the result of a few hours of research and coding.

Word Perfect's encryption algorithm in version 4.2 used an xor cipher. Since we didn't have any working binaries, the encryption/decryption algorithms were deciphered based on a paper written by Bennet, J[2] and a post by Helen Bergen. Using the information from these sources we were able to reconstruct the original algorithm. 

The algorithm is a simple xor symmetric algorithm. It uses a passphase as a key for ECB encryption mode as well as a second stream of sequenced bytes starting with keylength + 1 stored in a unsigned 8 bit integer, effectively: let sequence: u8 = keylength + 1. The sequence wraps as you would expect at 255 back to 0. 

This is effectively the same as breaking an XOR scheme since you can easily remove the sequence by "assuming" the keylenght will be between n..m, adding m-n potential xor streams to crack. 

There are a few perf improvements that could be made. However, this can crack a 10 char password almost instantly on a modern pc. 

#Usage

wp_crack --encrypt-file [some_file] --key [a_key] 

This will take the original file, encrypt it and output it in the current directory with the .enc extension.

wp_crack --decrypt-file [some_encrypted_file] --key [a_key]

This will take the encrypted file and decrypt it, leaving an unencrypted copy in the directory as .dec. 

wp_crack --decrypt-file [some_encrypted_file] --key " " --depth 5 --min 3 --max 12

This will attempt to crack the file. The depth is used for the freqency matrix. Min/max is the size of the potential password. Longer the key, the longer it takes.

##Resources
[1] Helen Bergen's postin 1990 on sci.form. https://groups.google.com/forum/#!topic/sci.crypt/PmxaYcslekE
[2] BENNETT, J (1987): Analysis of the encryption algorithm