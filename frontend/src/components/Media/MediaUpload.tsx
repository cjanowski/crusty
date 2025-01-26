import React, { useCallback } from 'react';
import {
    Box,
    Button,
    Text,
    useToast,
    VStack,
} from '@chakra-ui/react';
import { useDropzone } from 'react-dropzone';
import { media } from '../../api/client';
import { useMutation, useQueryClient } from '@tanstack/react-query';

export const MediaUpload: React.FC = () => {
    const toast = useToast();
    const queryClient = useQueryClient();

    const uploadMutation = useMutation({
        mutationFn: async (file: File) => {
            try {
                return await media.upload(file);
            } catch (error: any) {
                console.error('Upload error:', error);
                throw new Error(error.response?.data?.error || 'Failed to upload file');
            }
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['media'] });
            toast({
                title: 'File uploaded successfully',
                status: 'success',
                duration: 3000,
            });
        },
        onError: (error: Error) => {
            toast({
                title: 'Upload failed',
                description: error.message,
                status: 'error',
                duration: 3000,
            });
        },
    });

    const onDrop = useCallback((acceptedFiles: File[]) => {
        if (acceptedFiles.length > 0) {
            const file = acceptedFiles[0];
            console.log('Uploading file:', file.name, file.type, file.size);
            uploadMutation.mutate(file);
        }
    }, [uploadMutation]);

    const { getRootProps, getInputProps, isDragActive } = useDropzone({
        onDrop,
        maxFiles: 1,
        multiple: false,
    });

    return (
        <Box p={4}>
            <VStack spacing={4}>
                <Box
                    {...getRootProps()}
                    w="full"
                    h="200px"
                    border="2px dashed"
                    borderColor={isDragActive ? 'purple.500' : 'gray.200'}
                    borderRadius="md"
                    display="flex"
                    alignItems="center"
                    justifyContent="center"
                    cursor="pointer"
                    transition="all 0.2s"
                    _hover={{ borderColor: 'purple.500' }}
                    bg={isDragActive ? 'purple.50' : 'transparent'}
                >
                    <input {...getInputProps()} />
                    <VStack spacing={2}>
                        <Text color="gray.500">
                            {isDragActive
                                ? 'Drop the file here'
                                : 'Drag and drop a file here, or click to select'}
                        </Text>
                        {uploadMutation.isPending && (
                            <Text color="purple.500">Uploading...</Text>
                        )}
                    </VStack>
                </Box>
                <Button
                    colorScheme="purple"
                    onClick={() => document.querySelector('input')?.click()}
                    isDisabled={uploadMutation.isPending}
                >
                    Select File
                </Button>
            </VStack>
        </Box>
    );
}; 