import React from 'react';
import {
    Box,
    Grid,
    Text,
    IconButton,
    useToast,
    VStack,
} from '@chakra-ui/react';
import { DeleteIcon, DownloadIcon } from '@chakra-ui/icons';
import { media } from '../../api/client';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

interface MediaItem {
    id: string;
    filename: string;
    mime_type: string;
    created_at: string;
}

export const MediaGrid: React.FC = () => {
    const toast = useToast();
    const queryClient = useQueryClient();

    const { data: mediaItems, isLoading } = useQuery({
        queryKey: ['media'],
        queryFn: media.list,
    });

    const deleteMutation = useMutation({
        mutationFn: (id: string) => media.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['media'] });
            toast({
                title: 'File deleted successfully',
                status: 'success',
                duration: 3000,
            });
        },
        onError: () => {
            toast({
                title: 'Delete failed',
                description: 'Failed to delete file',
                status: 'error',
                duration: 3000,
            });
        },
    });

    const handleDownload = async (item: MediaItem) => {
        try {
            const blob = await media.get(item.id);
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = item.filename;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        } catch (error) {
            toast({
                title: 'Download failed',
                description: 'Failed to download file',
                status: 'error',
                duration: 3000,
            });
        }
    };

    if (isLoading) {
        return <Text>Loading...</Text>;
    }

    return (
        <Grid
            templateColumns="repeat(auto-fill, minmax(250px, 1fr))"
            gap={6}
            p={4}
        >
            {mediaItems?.map((item: MediaItem) => (
                <Box
                    key={item.id}
                    borderWidth="1px"
                    borderRadius="lg"
                    overflow="hidden"
                    p={4}
                    bg="white"
                    shadow="sm"
                >
                    <VStack align="stretch" spacing={3}>
                        <Text fontWeight="bold" noOfLines={1}>
                            {item.filename}
                        </Text>
                        <Text fontSize="sm" color="gray.500">
                            {new Date(item.created_at).toLocaleDateString()}
                        </Text>
                        <Box display="flex" justifyContent="space-between">
                            <IconButton
                                aria-label="Download file"
                                icon={<DownloadIcon />}
                                colorScheme="purple"
                                variant="ghost"
                                onClick={() => handleDownload(item)}
                            />
                            <IconButton
                                aria-label="Delete file"
                                icon={<DeleteIcon />}
                                colorScheme="red"
                                variant="ghost"
                                onClick={() => deleteMutation.mutate(item.id)}
                                isDisabled={deleteMutation.isPending}
                            />
                        </Box>
                    </VStack>
                </Box>
            ))}
        </Grid>
    );
}; 